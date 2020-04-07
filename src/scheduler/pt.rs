use super::*;

pub struct PTScheduler {
    inner: Mutex<PTSchedulerInner>,
}

struct PTSchedulerInner {
    max_time_slice: usize,
    infos: Vec<PTProcInfo>,
    active_queue: usize,
    queues: [VecDeque<Tid>; 5],
}

#[derive(Debug, Default, Copy, Clone)]
struct PTProcInfo {
    priority: u8,
    rest_slice: usize,
}

impl Scheduler for PTScheduler {
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
    fn remove(&self, tid: usize) {
        self.inner.lock().remove(tid)
    }
}

impl PTScheduler {
    pub fn new(max_time_slice: usize) -> Self {
        let inner = PTSchedulerInner {
            max_time_slice,
            infos: Vec::default(),
            active_queue: 0,
            queues: [VecDeque::default(), VecDeque::default(), VecDeque::default(), VecDeque::default(), VecDeque::default()],
        };
        PTScheduler {
            inner: Mutex::new(inner),
        }
    }
}

impl PTSchedulerInner {
    fn push(&mut self, tid: Tid) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        {
            let info = &mut self.infos[tid];
            if info.rest_slice == 0 {
                info.rest_slice = self.max_time_slice;
            }
            info!("in push, info.pri is {}", info.priority);
            info!("in push, tid is {}", tid);
            let priority = info.priority;
            self.queues[priority as usize].push_back(tid);
            info!("in push, info.rest_slice is {}", info.rest_slice);
            info!("in push, queues[pri].len() is {}", self.queues[priority as usize].len());
        }
        /* 
        {
            let info = &mut self.infos[tid];
            assert!(!info.present);
            info.present = true;
            let pri = info.priority;
            expand(&mut self.sch_infos, pri as usize);

            if info.rest_slice == 0 {
                info!("in push, info.pri is {}", info.priority);
                info.rest_slice = self.max_time_slice;
                info!("in push, info.rest_slice is {}", info.rest_slice);
            }
            self.sch_infos[pri as usize].push(*info);
        }
        self._list_add_before(tid, 0);
        trace!("pt push {}", tid - 1); */
    }

    fn pop(&mut self) -> Option<Tid> {
        info!("in pt pop()");
        self.active_queue = 0;
        info!("before init ret");
        let mut ret = match self.queues[0].pop_front() {
            Some(tid) => {
                info!("pop init result is {}", tid);
                return Some(tid);
            },
            None => {
                info!("pop ret init is none");
                self.queues[0].pop_front()
            }
        };
        info!("after init ret");
        for index in (0..5).rev() {
            info!("index is {}", index);
            if self.queues[index].len() > 0 {
                self.active_queue = index;
                ret = match self.queues[index].pop_front() {
                    Some(tid) => {
                        info!("pop result is {}", tid);
                        return Some(tid - 1)},
                    None => {
                        self.queues[index].pop_front()
                    }
                };
                break;
            }
        }
        info!("end pop");
        ret
    }

    fn tick(&mut self, current: Tid) -> bool {
        let current = current + 1;
        expand(&mut self.infos, current);

        let rest = &mut self.infos[current].rest_slice;
        if *rest > 0 {
            *rest -= 1;
        } else {
            warn!("current process rest_slice = 0, need reschedule")
        }
        info!("in tick, tid is {}, rest time is {}", current, rest);
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

    fn remove(&mut self, tid: Tid) {
        self._list_remove(tid + 1);
        //self.infos[tid + 1].present = false;
    }
}

impl PTSchedulerInner {
    fn _list_remove(&mut self, i: Tid) {
        let info = &mut self.infos[i];
        let priority = info.priority as usize;
        for index in 0..self.queues[priority].len() {
            if self.queues[priority][index] == i {
                self.queues[priority].remove(index);
                break;
            }
        }
        // let next = self.infos[i].next;
        // let prev = self.infos[i].prev;
        // self.infos[next].prev = prev;
        // self.infos[prev].next = next;
        // self.infos[i].next = 0;
        // self.infos[i].prev = 0;
    }
}
