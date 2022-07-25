#[derive(Clone, Copy, Debug)]
pub struct Resource(pub isize, pub isize);
pub const TEST_NUM: usize = 20 as usize;
pub struct BankerTester {
    pub all: [Resource; TEST_NUM],
    pub available: [Resource; TEST_NUM],
    max: [[Resource; TEST_NUM]; TEST_NUM],
    pub alloc: [[Resource; TEST_NUM]; TEST_NUM],
    pub need: [[Resource; TEST_NUM]; TEST_NUM],
    is_run: bool,
}

impl BankerTester {
    pub fn start(&mut self) {
        self.is_run = true;
    }
    pub fn is_run(&self) -> bool {
        self.is_run
    }
    pub fn is_available_mutex(&self, id: usize) -> bool {
        self.available[id].0 >= 1
    }
    pub fn is_available_semaphore(&self, id: usize) -> bool {
        self.available[id].1 >= 1
    }
    pub fn new() -> BankerTester{

        BankerTester {
            all: [Resource(0, 0); TEST_NUM],
            available: [Resource(0, 0); TEST_NUM],
            max: [[Resource(0, 0); TEST_NUM]; TEST_NUM],
            alloc: [[Resource(0, 0); TEST_NUM]; TEST_NUM],
            need: [[Resource(0, 0); TEST_NUM]; TEST_NUM],
            is_run: false,
        }
    }
    /// 
    pub fn add_mutex(&mut self, id: usize) {
        self.all[id].0 = 1;
    }
    /// 
    pub fn add_semaphore(&mut self, id: usize, count: isize) {
        self.all[id].1 = count;
    }
    /// 
    pub fn modify_mutex_need(&mut self, thread_id: usize, mutex_id: usize, count: isize) {
        self.need[thread_id][mutex_id].0 += count;
    }
    ///
    pub fn modify_semaphore_need(&mut self, thread_id: usize, resource_id: usize, count: isize) {
        self.need[thread_id][resource_id].1 += count;
    }
    ///
    pub fn clear_mutex_need(&mut self, thread_id: usize, mutex_id: usize) {
        self.need[thread_id][mutex_id].0 = 0;
    }
    ///
    pub fn clear_semaphore_need(&mut self, thread_id: usize, resource_id: usize) {
        self.need[thread_id][resource_id].1 = 0;
    }
    ///
    pub fn modify_mutex_alloc(&mut self, thread_id: usize, mutex_id: usize, count: isize) {
        self.alloc[thread_id][mutex_id].0 += count;
    }
    ///
    pub fn modify_semaphore_alloc(&mut self, thread_id: usize, resource_id: usize, count: isize) {
        self.alloc[thread_id][resource_id].1 += count;
    }
    pub fn clear_mutex_alloc(&mut self, thread_id: usize, mutex_id: usize) {
        self.alloc[thread_id][mutex_id].0 = 0;
    }
    ///
    pub fn clear_semaphore_alloc(&mut self, thread_id: usize, resource_id: usize) {
        self.alloc[thread_id][resource_id].1 = 0;
    }
    /// 
    pub fn safety_check(&mut self) -> isize {
        // self.available = {
        //     let mut temp = self.all;
        //     for i in 0..self.all.len() {
        //         let mut num_mutex = 0;
        //         for j in 0..self.alloc.len() {
        //             num_mutex += self.alloc[j][i].0;
        //         }
        //         let mut num_sem = 0;
        //         for j in 0..self.alloc.len() {
        //             num_sem += self.alloc[j][i].1;
        //         }
        //         temp[i] = Resource(self.all[i].0 - num_mutex, self.all[i].1 - num_sem);
        //     }
        //     temp
        // };
        let mut work = self.available;
        let mut finish = [false; TEST_NUM];
        // println!("available {:?} \n need {:?} \n alloc {:?}", work, self.need, self.alloc);
        for count in 0..finish.len() {
            for thread_id in 0..finish.len() {
                if finish[thread_id] {
                    continue;
                }
                let mut flag = true;
                for source_id in 0..work.len() {
                    if self.need[thread_id][source_id].0 > work[source_id].0 {
                        flag = false;
                        break;
                    }
                }
                if flag == true {
                    for source_id in 0..work.len() {
                        work[source_id].0 += self.alloc[thread_id][source_id].0;
                    }
                    finish[thread_id] = true;
                }
            }
        }
        
        let finish_num = finish.iter().filter(| &x | *x == true).count();
        // println!("available {:?} need {:?} alloc {:?}", work, self.need, self.alloc);
        // println!("MutexChecker Finish_num = {}", finish_num);
        if finish_num != finish.len() {
            return -1;
        }

        let mut finish = [false; TEST_NUM];

        for count in 0..finish.len() {
            for thread_id in 0..finish.len() {
                if finish[thread_id] {
                    continue;
                }
                let mut flag = true;
                for source_id in 0..work.len() {
                    if self.need[thread_id][source_id].1 > work[source_id].1 {
                        flag = false;
                        break;
                    }
                }
                if flag == true {
                    for source_id in 0..work.len() {
                        work[source_id].1 += self.alloc[thread_id][source_id].1;
                    }
                    finish[thread_id] = true;
                }
            }
        }
        let finish_num = finish.iter().filter(| &x | *x == true).count();
        // println!("SemChecker Finish_num = {}", finish_num);
        if finish_num != finish.len() {
            return -1;
        }
        return 0;
    }
}
