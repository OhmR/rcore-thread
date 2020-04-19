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
    tick_num: u8,
    start_flag: bool,
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
    }

    fn pop(&mut self) -> Option<Tid> {
        info!("in pt pop()");
        self.active_queue = 0;
        info!("before init ret");
        let mut ret = None;
        info!("after init ret");
        for index in (0..5).rev() {
            info!("index is {}", index);
            if self.queues[index].len() > 0 {
                self.active_queue = index;
                ret = match self.queues[index].pop_front() {
                    Some(tid) => {
                        info!("pop result is {}", tid);
                        Some(tid - 1)},
                    None => {
                        self.queues[index].pop_front()
                    }
                };
                break;
            }
        }
        info!("end pop");
        if ret == None {
            info!("pop result is None");
        }
        ret
    }

    fn tick(&mut self, current: Tid) -> bool {
        let current = current + 1;
        expand(&mut self.infos, current);

        let info = &mut self.infos[current];
        if info.start_flag {
            info.tick_num += 1;
        }

        //let rest = &mut self.infos[current].rest_slice;
        if info.rest_slice > 0 {
            info.rest_slice -= 1;
        } else {
            warn!("current process rest_slice = 0, need reschedule")
        }
        info!("in tick, tid is {}, rest time is {}, tick num is {}", current, info.rest_slice, info.tick_num);
        info.rest_slice == 0
    }

    fn set_priority(&mut self, tid: Tid, priority: u8) {
        info!("before set info.pri in schedule, pri is {}", priority);
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        if priority > 5 {
            info.priority = 4;
        } else {
            info.priority = priority - 1;
        }
        info!("after set info.pri in schedule, info.pri is {}", info.priority);
    }

    fn cal_priority(&mut self, priority: u8) -> u8 {
        if priority > 5 {
            5
        } else {
            priority
        }
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
    }
}

impl PTSchedulerInner {
    fn start(&mut self, tid: usize) {
        info!("{} start tick num", tid);
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.start_flag = true;
        info.tick_num = 0;
    }
    fn get_tick(&mut self, tid: usize) -> u8 {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.tick_num
    }
    fn end(&mut self, tid: usize) {
        info!("{} end tick num", tid);
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        let info = &mut self.infos[tid];
        info.start_flag = false;
        info.tick_num = 0;
    }
}