use alloc::{collections::BinaryHeap, vec::Vec, collections::VecDeque};

use log::*;
use spin::Mutex;

pub use self::o1::O1Scheduler;
pub use self::rr::RRScheduler;
pub use self::pt::PTScheduler;
pub use self::stride::StrideScheduler;
pub use self::work_stealing::WorkStealingScheduler;

mod pt;
mod o1;
mod rr;
mod stride;
mod work_stealing;

type Tid = usize;

/// The scheduler for a ThreadPool
pub trait Scheduler: 'static {
    /// Push a thread to the back of ready queue.
    fn push(&self, tid: Tid);
    /// Select a thread to run, pop it from the queue.
    fn pop(&self, cpu_id: usize) -> Option<Tid>;
    /// Got a tick from CPU.
    /// Return true if need reschedule.
    fn tick(&self, current_tid: Tid) -> bool;
    /// Set priority of a thread.
    fn set_priority(&self, tid: Tid, priority: u8);
    /// calculate priority of a thread.
    fn cal_priority(&self, priority: u8) -> u8;
    /// remove a thread in ready queue.
    fn remove(&self, tid: Tid);
    /// Set start calculate time slice
    fn start(&self, tid:Tid);
    /// Get tick num;
    fn get_tick(&self, tid:Tid) -> u8;
    /// End and reset tick time
    fn end(&self, tid:Tid);
}

fn expand<T: Default + Clone>(vec: &mut Vec<T>, id: usize) {
    let len = vec.len();
    vec.resize(len.max(id + 1), T::default());
}
