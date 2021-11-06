use std::slice;

mod bindings;
mod solver;

pub use solver::DlxSolver;

use bindings::*;

#[repr(transparent)]
struct DlxMatrix {
    handle: dlx_t,
}

impl DlxMatrix {
    #[inline]
    pub fn new() -> Self {
        Self {
            handle: unsafe { dlx_new() },
        }
    }

    #[inline]
    pub fn rows(&self) -> i32 {
        unsafe { dlx_rows(self.handle) }
    }

    #[inline]
    pub fn cols(&self) -> i32 {
        unsafe { dlx_rows(self.handle) }
    }

    #[inline]
    pub fn set(&mut self, row: i32, col: i32) {
        unsafe {
            dlx_set(self.handle, row, col);
        }
    }

    #[inline]
    pub fn mark_optional(&mut self, col: i32) {
        unsafe {
            dlx_mark_optional(self.handle, col);
        }
    }

    #[inline]
    pub fn remove_row(&mut self, row: i32) -> Result<(), ()> {
        unsafe {
            if dlx_remove_row(self.handle, row) == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    #[inline]
    pub fn pick_row(&mut self, row: i32) -> Result<(), ()> {
        unsafe {
            if dlx_pick_row(self.handle, row) == 0 {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn solve_all(&mut self) -> Vec<Vec<i32>> {
        unsafe {
            let data_ptr: *mut Vec<Vec<i32>> = Box::into_raw(Box::new(Vec::new()));
            dlx_forall_cover(self.handle, Some(dlx_solve_found_cb), data_ptr as *mut _);
            *Box::from_raw(data_ptr)
        }
    }

    pub fn solve_first(&mut self) -> Option<Vec<i32>> {
        unsafe {
            let data_ptr: *mut Vec<Vec<i32>> = Box::into_raw(Box::new(Vec::new()));
            dlx_first_cover(self.handle, Some(dlx_solve_found_cb), data_ptr as *mut _);
            (*Box::from_raw(data_ptr)).pop()
        }
    }
}

#[allow(dead_code)]
unsafe extern "C" fn dlx_solve_found_cb(data: *mut std::os::raw::c_void, arr: *mut i32, size: i32) {
    let data_ptr = data as *mut Vec<Vec<i32>>;
    let data = &mut *data_ptr;

    let solution = slice::from_raw_parts(arr, size as usize).to_vec();
    println!("Solution: {:?}", solution);
    data.push(solution);
}

impl Drop for DlxMatrix {
    fn drop(&mut self) {
        unsafe {
            dlx_clear(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dlx_solve() {
        let mut dlx = DlxMatrix::new();

        //   | 0 | 1 | 2
        // --+---+---+---
        // 0 | 1 | 0 | 1
        // --+---+---+---
        // 1 | 0 | 1 | 1
        // --+---+---+---
        // 2 | 0 | 1 | 0
        // --+---+---+---
        // 3 | 1 | 1 | 0

        dlx.set(0, 0);
        dlx.set(0, 2);
        dlx.set(1, 1);
        dlx.set(1, 2);
        dlx.set(2, 1);
        dlx.set(3, 0);
        dlx.set(3, 1);

        dlx.mark_optional(2);

        let solutions = dlx.solve_all();

        assert_eq!(solutions.len(), 2);
        assert_eq!(solutions[0], vec![0, 2]);
        assert_eq!(solutions[1], vec![3]);
    }
}
