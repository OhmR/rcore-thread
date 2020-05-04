use super::*;

pub struct RRScheduler {
    inner: Mutex<RRSchedulerInner>,
}

struct RRSchedulerInner {
    max_time_slice: usize,
    infos: Vec<RRProcInfo>,
}

#[derive(Debug, Default, Copy, Clone)]
struct RRProcInfo {
    present: bool,
    priority: u8,
    rest_slice: usize,
    prev: Tid,
    next: Tid,
    tick_num: u8,
    start_flag: bool,
    success: bool,
}

impl Scheduler for RRScheduler {
    fn push(&self, tid: usize) {
        self.inner.lock().push(tid);
    }
    fn pop(&self, _cpu_id: usize) -> Option<usize> {
        self.inner.lock().pop()
    }
    fn tick(&self, current_tid: usize) -> bool {
        self.inner.lock().tick(current_tid)
    }
    fn set_priority(&self, _tid: usize, _priority: u8) {
        self.inner.lock().set_priority(_tid, _priority)
    }
    fn cal_priority(&self, _priority: u8) -> u8 {
        self.inner.lock().cal_priority(_priority)
    }
    fn remove(&self, tid: usize) {
        self.inner.lock().remove(tid)
    }
    fn start(&self, tid: usize) {
        self.inner.lock().start(tid)
    }
    fn get_tick(&self, tid: usize) -> u8 {
        self.inner.lock().get_tick(tid)
    }
    fn end(&self, tid: usize) {
        self.inner.lock().end(tid)
    }
    fn set_success(&self, tid:usize, value: bool) {
        self.inner.lock().set_success(tid, value)
    }
    fn get_success(&self, tid:usize) -> bool {
        self.inner.lock().get_success(tid)
    }
    fn reset_slice(&self, tid:usize) {
        self.inner.lock().reset_slice(tid)
    }
}

impl RRScheduler {
    pub fn new(max_time_slice: usize) -> Self {
        let inner = RRSchedulerInner {
            max_time_slice,
            infos: Vec::default(),
        };
        RRScheduler {
            inner: Mutex::new(inner),
        }
    }
}

impl RRSchedulerInner {
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        {
            let info = &mut self.infos[tid];
            assert!(!info.present);
            info.present = true;
            if info.rest_slice == 0 {
                info!("in push, info.pri is {}", info.priority);
                info.rest_slice = self.max_time_slice * info.priority as usize;
                info!("in push, info.rest_slice is {}", info.rest_slice);
            }
        }
        self._list_add_before(tid, 0);
        trace!("rr push {}", tid - 1);
    }


    // fn push_pri(&mut self, tid: Tid, priority: u8) {
    //     let tid = tid + 1;
    //     expand(&mut self.infos, tid);
    //     {
    //         let info = &mut self.infos[tid];
    //         assert!(!info.present);
    //         info.present = true;
    //         info.priority = priority;
    //         if info.rest_slice == 0 {
    //             info!("in push, info.pri is {}", info.priority);
    //             info.rest_slice = self.max_time_slice * info.priority as usize;
    //             info!("in push, info.rest_slice is {}", info.rest_slice);
    //         }
    //     }
    //     self._list_add_before(tid, 0);
    //     trace!("rr push {}", tid - 1);
    // }

    fn pop(&mut self) -> Option<Tid> {
        let ret = match self.infos[0].next {
            0 => None,
            tid => {
                self.infos[tid].present = false;
                self._list_remove(tid);
                Some(tid - 1)
            }
        };
        trace!("rr pop {:?}", ret);
        ret
    }

    fn tick(&mut self, current: Tid) -> bool {
        let current = current + 1;
        expand(&mut self.infos, current);
        assert!(!self.infos[current].present);

        let info = &mut self.infos[current];
        if info.start_flag {
            info.tick_num += 1;
        }

        let rest = &mut self.infos[current].rest_slice;
        if *rest > 0 {
            *rest -= 1;
        } else {
            warn!("current process rest_slice = 0, need reschedule")
        }
        *rest == 0
    }

    fn set_priority(&mut self, tid: Tid, priority: u8) {
        info!("before set info.pri in schedule, pri is {}", priority);
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.priority = priority;
        info!("after set info.pri in schedule, info.pri is {}", info.priority);
    }

    fn cal_priority(&mut self, priority: u8) -> u8 {
        priority
    }

    fn remove(&mut self, tid: Tid) {
        self._list_remove(tid + 1);
        self.infos[tid + 1].present = false;
        info!("remove thread {}", tid);
        // info!("current length is {}", self.infos.len());
    }
}

impl RRSchedulerInner {
    fn _list_add_before(&mut self, i: Tid, at: Tid) {
        let prev = self.infos[at].prev;
        self.infos[i].next = at;
        self.infos[i].prev = prev;
        self.infos[prev].next = i;
        self.infos[at].prev = i;
    }
    fn _list_add_after(&mut self, i: Tid, at: Tid) {
        let next = self.infos[at].next;
        self._list_add_before(i, next);
    }
    fn _list_remove(&mut self, i: Tid) {
        let next = self.infos[i].next;
        let prev = self.infos[i].prev;
        self.infos[next].prev = prev;
        self.infos[prev].next = next;
        self.infos[i].next = 0;
        self.infos[i].prev = 0;
    }
}

impl RRSchedulerInner {
    fn start(&mut self, tid: usize) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.start_flag = true;
        info.tick_num = 0;
        info.success = false;
    }
    fn get_tick(&mut self, tid: usize) -> u8 {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.tick_num
    }
    fn end(&mut self, tid: usize) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.start_flag = false;
        info.tick_num = 0;
    }
    fn set_success(&mut self, tid: usize, value: bool) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.success = value;
    }
    fn get_success(&mut self, tid: usize) -> bool {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.success
    }
    fn reset_slice(&mut self, tid: usize ) {
        info!("{} reset rest slice", tid);
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.rest_slice = 0;
    }
}