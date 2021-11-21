// Structure based on:
// http://pds25.egloos.com/pds/201504/21/98/RectangleBinPack.pdf

#[derive(Clone, Debug)]
struct Rect {
    x: usize,
    y: usize,  // NOTE: bottom
    w: usize,
    h: usize,
}

impl Rect {
    fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Rect {
            x: x, y: y, w: w, h: h
        }
    }

    fn top(&self) -> usize {
        self.y + self.h
    }

    fn bottom(&self) -> usize {
        self.y
    }

    fn left(&self) -> usize {
        self.x
    }

    fn right(&self) -> usize {
        self.x + self.w
    }

    fn fits(&self, width: usize, height: usize) -> bool {
        width <= self.w && height <= self.h
    }

    fn fully_contains(&self, other: &Rect) -> bool {
        other.left() >= self.left() &&
        other.right() <= self.right() &&
        other.bottom() >= self.bottom() &&
        other.top() <= self.top()
    }

    fn intersects(&self, other: &Rect) -> bool {
        let no_intersect =
            self.right() <= other.left() || self.left() >= other.right() ||
            self.bottom() >= other.top() || self.top() <= other.bottom();
        !no_intersect
    }
}

#[derive(Clone)]
pub struct MaxRects {
    width: usize,
    height: usize,
    // Free spaces.
    free: Vec<Rect>,
}

impl MaxRects {
    pub fn new(width: usize, height: usize) -> Self {
        MaxRects {
            width: width, height: height,
            free: vec![Rect::new(0, 0, width, height)],
        }
    }

    fn split_at(&mut self, index: usize, splitter_width: usize, splitter_height: usize) {
        let to_split = self.free[index].clone();
        // Regular updates to our free spaces will automatically take care of splitting the above in two.
        self.update_overlaps(&Rect::new(to_split.x, to_split.y, splitter_width, splitter_height));
        self.remove_redundancy();
    }

    fn update_overlaps(&mut self, not_free: &Rect) {
        let mut i = 0;
        let mut added_splits = 0;
        while i < self.free.len() - added_splits {
            if self.free[i].intersects(not_free) {
                let to_split = self.free[i].clone();
                if not_free.left() > to_split.left() {  // left split
                    self.free.push(Rect::new(to_split.left(), to_split.bottom(),
                        not_free.left() - to_split.left(), to_split.h));
                    added_splits += 1;
                }
                if not_free.right() < to_split.right() {  // right split
                    self.free.push(Rect::new(not_free.right(), to_split.bottom(),
                        to_split.right() - not_free.right(), to_split.h));
                    added_splits += 1;
                }
                if not_free.top() < to_split.top() {  // top split
                    self.free.push(Rect::new(to_split.left(), not_free.top(),
                        to_split.w, to_split.top() - not_free.top()));
                    added_splits += 1;
                }
                if not_free.bottom() > to_split.bottom() {  // bottom split
                    self.free.push(Rect::new(to_split.left(), to_split.bottom(),
                        to_split.w, not_free.bottom() - to_split.bottom()));
                    added_splits += 1;
                }
                self.free.swap_remove(i);
                if added_splits > 0 {
                    added_splits -= 1;
                    // We just swapped with a created split, we can skip it, it won't intersect.
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    fn remove_redundancy(&mut self) {
        let mut i = 0;
        while i < self.free.len() {
            let mut j = i + 1;
            while j < self.free.len() {
                if self.free[i].fully_contains(&self.free[j]) {
                    self.free.swap_remove(j);
                } else if self.free[j].fully_contains(&self.free[i]) {
                    self.free.swap_remove(i);
                    j = i + 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    // If target fits, returns (x, y) position of the bottom-left corner where it would fit.
    // Will pick the lowest left-aligned fit.
    pub fn bottom_left_insert(&mut self, width: usize, height: usize) -> Option<(usize, usize)> {
        let mut best_index = None;
        for (i, rect) in self.free.iter().enumerate() {
            if rect.fits(width, height) {
                let better_candidate = match best_index {
                    None => true,
                    Some(index) => {
                        let best_rect: &Rect = &self.free[index];
                        rect.y < best_rect.y || (rect.y == best_rect.y && rect.x < best_rect.x)
                    }
                };
                if better_candidate {
                    best_index = Some(i);
                }
            }
        }
        let best_index = best_index?;
        let rect = &self.free[best_index];
        let (x, y) = (rect.x, rect.y);
        self.split_at(best_index, width, height);
        Some((x, y))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_left_insert_empty_fits() {
        let mut m = MaxRects::new(10, 20);
        let (x, y) = m.bottom_left_insert(5, 7).unwrap();
        assert!((x, y) == (0, 0));
    }

    #[test]
    fn bottom_left_insert_empty_no_fit() {
        let mut m = MaxRects::new(10, 20);
        assert!(m.bottom_left_insert(30, 40).is_none());
    }

    #[test]
    fn bottom_left_insert_2_splits() {
        let mut m = MaxRects::new(10, 20);
        let (x, y) = m.bottom_left_insert(2, 3).unwrap();
        assert!((x, y) == (0, 0));
        assert!(m.free.len() == 2);
    }
}