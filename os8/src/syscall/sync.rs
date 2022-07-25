use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use super::thread::{sys_gettid};
use super::process::{sys_exit};

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

// LAB5 HINT: you might need to maintain data structures used for deadlock detection
// during sys_mutex_* and sys_semaphore_* syscalls
pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        process_inner.banker_tester.available[id].0 = 1;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        let id = (process_inner.mutex_list.len() - 1) as usize;
        process_inner.banker_tester.available[id].0 = 1;
        process_inner.mutex_list.len() as isize - 1
    }
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tid = sys_gettid() as usize;
    process_inner.banker_tester.need[tid][mutex_id].0 += 1;
    // process_inner.banker_tester.modify_mutex_need(sys_gettid() as usize, mutex_id, 1 as isize);
    if process_inner.banker_tester.is_run() && process_inner.banker_tester.safety_check() != 0 {
        process_inner.banker_tester.need[tid][mutex_id].1 = 0;
        process_inner.banker_tester.alloc[tid][mutex_id].1 = 0;
        // println!("thread_id = {} mutex = {} mutex_safety_check fail!", sys_gettid(), mutex_id);
        drop(process_inner);
        drop(process);
        -0xDEAD as isize
    } else {
        if process_inner.banker_tester.available[mutex_id].0 > 0 {
            process_inner.banker_tester.need[tid][mutex_id].0 -= 1;
            process_inner.banker_tester.alloc[tid][mutex_id].0 += 1;
            process_inner.banker_tester.available[mutex_id].0 -= 1;
        }
        drop(process_inner);
        drop(process);
        mutex.lock();
        0
    }
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let tid = sys_gettid() as usize;
    if process_inner.banker_tester.alloc[tid][mutex_id].0 > 0 {
        process_inner.banker_tester.alloc[tid][mutex_id].0 -= 1;
        process_inner.banker_tester.available[mutex_id].0 += 1;
    } else {
        process_inner.banker_tester.need[tid][mutex_id].0 -= 1;
    }
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        // process_inner.banker_tester.add_semaphore(id, res_count as isize);
        process_inner.banker_tester.available[id].1 = res_count as isize;
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        let id = (process_inner.semaphore_list.len() - 1) as usize;
        // process_inner.banker_tester.add_semaphore(id, res_count as isize);
        process_inner.banker_tester.available[id].1 = res_count as isize;
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let tid = sys_gettid() as usize;
    if process_inner.banker_tester.alloc[tid][sem_id].1 > 0 {
        process_inner.banker_tester.alloc[tid][sem_id].1 -= 1;
        process_inner.banker_tester.available[sem_id].1 += 1;
    } else {
        process_inner.banker_tester.need[tid][sem_id].1 -= 1;
    }
    // process_inner.banker_tester.modify_semaphore_need(sys_gettid() as usize, sem_id, -1 as isize);
    drop(process_inner);
    drop(process);
    sem.up();
    0
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    let tid = sys_gettid() as usize;
    process_inner.banker_tester.need[tid][sem_id].1 += 1;
    // println!("TID = {} , sem down sem_id = {}", tid, sem_id);
    if process_inner.banker_tester.is_run() && process_inner.banker_tester.safety_check() != 0 {
        process_inner.banker_tester.need[tid][sem_id].1 = 0;
        process_inner.banker_tester.alloc[tid][sem_id].1 = 0;
        drop(process_inner);
        drop(process);
        -0xDEAD
    } else {
        if process_inner.banker_tester.available[sem_id].1 > 0 {
            process_inner.banker_tester.need[tid][sem_id].1 -= 1;
            process_inner.banker_tester.alloc[tid][sem_id].1 += 1;
            process_inner.banker_tester.available[sem_id].1 -= 1;
        }
        drop(process_inner);
        drop(process);
        sem.down();
        0
    }
}

pub fn sys_condvar_create(_arg: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}

// LAB5 YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    if _enabled > 1 || _enabled < 0 {
        return -1;
    } else {
        let process = current_process();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.banker_tester.start();
        return 0;
    }
}
