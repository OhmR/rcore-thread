//! Stride scheduler
//!
//! Each task is assigned a priority. Each task has a running stride.
//! The task with least stride is selected to run.
//! When a task is rescheduled, its stride is added to proportional to 1 / priority.

use super::*;
use core::cmp::{Ordering, Reverse};

pub struct StrideScheduler {
    inner: Mutex<StrideSchedulerInner>,
}

pub struct StrideSchedulerInner {
    max_time_slice: usize,
    infos: Vec<StrideProcInfo>,
    queue: BinaryHeap<Reverse<(Stride, Tid)>>, // It's max heap, so use Reverse
}

#[derive(Debug, Default, Copy, Clone)]
struct StrideProcInfo {
    present: bool,
    rest_slice: usize,
    stride: Stride,
    priority: u8,
}

const BIG_STRIDE: Stride = Stride(0x7FFFFFFF);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct Stride(u32);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Stride {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 == other.0 {
            Ordering::Equal
        } else {
            let sub = other.0.overflowing_sub(self.0).0;
            if sub <= BIG_STRIDE.0 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    }
}

impl StrideProcInfo {
    fn pass(&mut self) {
        let pass = if self.priority == 0 {
            BIG_STRIDE.0
        } else {
            BIG_STRIDE.0 / self.priority as u32
        };
        self.stride = Stride(self.stride.0.overflowing_add(pass).0);
    }
}

impl Scheduler for StrideScheduler {
    fn push(&self, tid: usize) {
        self.inner.lock().push(tid);
    }
    fn pop(&self, _cpu_id: usize) -> Option<usize> {
        self.inner.lock().pop()
    }
    fn tick(&self, current_tid: usize) -> bool {
        self.inner.lock().tick(current_tid)
    }
    fn set_priority(&self, tid: usize, priority: u8) {
        self.inner.lock().set_priority(tid, priority);
    }
    fn cal_priority(&self, _priority: u8) -> u8 {
        self.inner.lock().cal_priority(_priority)
    }
    fn remove(&self, tid: usize) {
        self.inner.lock().remove(tid);
    }
}

impl StrideScheduler {
    pub fn new(max_time_slice: usize) -> Self {
        let inner = StrideSchedulerInner {
            max_time_slice,
            infos: Vec::default(),
            queue: BinaryHeap::default(),
        };
        StrideScheduler {
            inner: Mutex::new(inner),
        }
    }
}

impl StrideSchedulerInner {
    fn push(&mut self, tid: Tid) {
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.present = true;
        if info.rest_slice == 0 {
            info.rest_slice = self.max_time_slice;
        }
        self.queue.push(Reverse((info.stride, tid)));
        trace!("stride push {}", tid);
    }

    fn pop(&mut self) -> Option<Tid> {
        let ret = self.queue.pop().map(|Reverse((_, tid))| tid);
        if let Some(tid) = ret {
            let info = &mut self.infos[tid];
            if !info.present {
                return self.pop();
            }
            let old_stride = info.stride;
            info.pass();
            let stride = info.stride;
            info.present = false;
            trace!("stride {} {:#x} -> {:#x}", tid, old_stride.0, stride.0);
        }
        trace!("stride pop {:?}", ret);
        ret
    }

    fn tick(&mut self, current: Tid) -> bool {
        expand(&mut self.infos, current);
        assert!(!self.infos[current].present);

        let rest = &mut self.infos[current].rest_slice;
        if *rest > 0 {
            *rest -= 1;
        } else {
            warn!("current process rest_slice = 0, need reschedule")
        }
        *rest == 0
    }

    fn set_priority(&mut self, tid: Tid, priority: u8) {
        self.infos[tid].priority = priority;
        trace!("stride {} priority = {}", tid, priority);
    }

    fn cal_priority(&mut self, priority: u8) -> u8 {
        priority
    }

    fn remove(&mut self, tid: Tid) {
        self.infos[tid].present = false;
    }
}
