use std::{
    fs::{File, OpenOptions},
    mem,
    os::fd::AsRawFd,
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use io_uring::{opcode, types, IoUring};
use libc::{read, size_t, stat64, write, POSIX_FADV_SEQUENTIAL};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Options {
    #[command(subcommand)]
    method: ProcessType,

    #[arg(
        short = 'p',
        long = "path",
        help = "Specifies the path for the source and target. If not provided, the current directory is used."
    )]
    path: Option<String>,

    #[arg(
        short = 'f',
        long = "filename",
        help = "Specifies the name of the target file. If not provided, 'chunck' is used."
    )]
    filename: Option<String>,

    #[arg(
        short = 'b',
        long = "blocksize",
        help = "Specifies the block size. If not provided, the filesystem's fstat report multiplied by 32 is used."
    )]
    block_size: Option<i64>,
}

#[derive(Subcommand, Debug)]
enum ProcessType {
    /// Uses read and write system calls
    Synchronous,
    /// Uses uring with SQPOLL
    IoUring,
}

fn main() {
    let opts: Options = Options::parse();

    let path: PathBuf = PathBuf::from(opts.path.unwrap_or(".".to_string()));
    let source_filename = opts.filename.unwrap_or("chunk".to_string());
    let target_filename = format!("{}_copy", source_filename);

    let mut source_path = path.clone();
    source_path.push(source_filename);

    let mut target_path = path.clone();
    target_path.push(target_filename);

    let blocksize = opts.block_size.unwrap_or(calculate_blocksize(&source_path)) * 32;

    match opts.method {
        ProcessType::Synchronous => {
            CopyLibc::copy_with(&source_path, &target_path, blocksize as usize);
        }
        ProcessType::IoUring => {
            CopyIoUring::copy_with(&source_path, &target_path, blocksize as usize);
        }
    }
}

/**
 * Get the blocksize of the filesystem.
 */
fn calculate_blocksize(source_path: &PathBuf) -> i64 {
    let source_file = File::open(source_path).expect("Failed to open source file");
    unsafe {
        let mut raw_stat = mem::zeroed::<stat64>();
        libc::fstat64(source_file.as_raw_fd(), &mut raw_stat as *mut _);
        raw_stat.st_blksize
    }
}

pub trait TurboCopy {
    fn copy_with(
        source_path: &std::path::PathBuf,
        target_path: &std::path::PathBuf,
        blocksize: usize,
    );
}

pub struct CopyLibc;

impl TurboCopy for CopyLibc {
    fn copy_with(source_path: &PathBuf, target_path: &PathBuf, blocksize: usize) {
        let source = OpenOptions::new().read(true).open(source_path).unwrap();
        let target = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(target_path)
            .unwrap();

        let mut buffer = Vec::with_capacity(blocksize);

        let source = source.as_raw_fd();
        let target = target.as_raw_fd();
        let buffer = buffer.as_mut_ptr();

        unsafe {
            libc::posix_fadvise64(source, 0, 0, POSIX_FADV_SEQUENTIAL);
        }

        let mut last_read = -1;

        while last_read != 0 {
            last_read = unsafe { read(source, buffer, blocksize) };
            unsafe {
                write(target, buffer, last_read as size_t);
            };
        }
    }
}

pub struct CopyIoUring;

impl TurboCopy for CopyIoUring {
    fn copy_with(source_path: &PathBuf, target_path: &PathBuf, blocksize: usize) {
        let source = OpenOptions::new().read(true).open(source_path).unwrap();
        let target = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(target_path)
            .unwrap();

        let mut ring: IoUring<io_uring::squeue::Entry, io_uring::cqueue::Entry> =
            IoUring::builder().setup_sqpoll(500).build(2).unwrap();

        let mut last_read = -1;
        let mut offset = 0;

        let mut buf = Vec::with_capacity(blocksize);
        let (submitter, mut submission_queue, mut completion_queue) = ring.split();

        while last_read != 0 {
            let r_entry = opcode::Read::new(
                types::Fd(source.as_raw_fd()),
                buf.as_mut_ptr(),
                blocksize as u32,
            )
            .offset(offset)
            .build();
            unsafe {
                let _ = submission_queue.push(&r_entry);
            }

            submission_queue.sync();
            completion_queue.sync();
            submitter.submit_and_wait(1).unwrap();
            submission_queue.sync();
            completion_queue.sync();

            match completion_queue.next() {
                None => {}
                Some(e) => {
                    last_read = e.result();
                }
            }
            let w_entry = opcode::Write::new(
                types::Fd(target.as_raw_fd()),
                buf.as_mut_ptr(),
                last_read as u32,
            )
            .offset(offset)
            .build();
            unsafe {
                let _ = submission_queue.push(&w_entry);
            }

            submission_queue.sync();
            completion_queue.sync();
            submitter.submit_and_wait(1).unwrap();
            submission_queue.sync();
            completion_queue.sync();
            completion_queue.next();

            offset += last_read as u64;
        }
    }
}
