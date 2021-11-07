use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type Idx = usize;

pub type OwnedNode = Rc<RefCell<Node>>;
pub type WeakNode = Weak<RefCell<Node>>;

#[derive(Clone, Debug)]
pub enum NodeAttr {
    Root,
    Header(usize, bool), // Column size
    Cell(WeakNode),      // Header reference
}

#[derive(Debug)]
pub struct Node {
    me: WeakNode,

    up: WeakNode,
    down: WeakNode,
    left: WeakNode,
    right: WeakNode,

    idx: Idx,

    attr: NodeAttr,
}

impl Node {
    pub fn new(idx: Idx, attr: NodeAttr) -> OwnedNode {
        let slf = Rc::new(RefCell::new(Self {
            me: Weak::new(),
            up: Weak::new(),
            down: Weak::new(),
            left: Weak::new(),
            right: Weak::new(),
            idx,
            attr,
        }));

        {
            let mut node = (*slf).borrow_mut();
            let node_ref = Rc::downgrade(&slf);
            node.up = node_ref.clone();
            node.down = node_ref.clone();
            node.left = node_ref.clone();
            node.right = node_ref.clone();
            node.me = node_ref;
        }

        slf
    }

    #[inline(always)]
    pub fn new_root() -> OwnedNode {
        Self::new(0, NodeAttr::Root)
    }

    #[inline(always)]
    pub fn new_header(col: Idx) -> OwnedNode {
        Self::new(col, NodeAttr::Header(0, false))
    }

    #[inline(always)]
    pub fn new_cell(header: &WeakNode, row: Idx) -> OwnedNode {
        Self::new(row, NodeAttr::Cell(header.clone()))
    }

    #[inline(always)]
    pub fn idx(&self) -> Idx {
        self.idx
    }

    #[inline(always)]
    pub fn up(&self) -> WeakNode {
        self.up.clone()
    }

    #[inline(always)]
    pub fn down(&self) -> WeakNode {
        self.down.clone()
    }

    #[inline(always)]
    pub fn left(&self) -> WeakNode {
        self.left.clone()
    }

    #[inline(always)]
    pub fn right(&self) -> WeakNode {
        self.right.clone()
    }

    #[inline(always)]
    pub fn header(&self) -> WeakNode {
        match &self.attr {
            NodeAttr::Cell(h) => h.clone(),
            _ => self.me.clone(),
        }
    }

    #[inline(always)]
    pub fn size(&self) -> Option<usize> {
        match self.attr {
            NodeAttr::Header(s, _) => Some(s),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn inc_size(&mut self) {
        if let NodeAttr::Header(c, o) = self.attr {
            self.attr = NodeAttr::Header(c + 1, o);
        }
    }

    #[inline(always)]
    pub fn dec_size(&mut self) {
        if let NodeAttr::Header(c, o) = self.attr {
            self.attr = NodeAttr::Header(c - 1, o);
        }
    }

    pub fn optional(&self) -> bool {
        match &self.attr {
            NodeAttr::Header(_, o) => *o,
            NodeAttr::Cell(h) => {
                let h = h.upgrade().unwrap();
                let h = (*h).borrow();
                h.optional()
            }
            _ => false,
        }
    }

    pub fn set_optional(&mut self) {
        if let NodeAttr::Header(s, _) = self.attr {
            self.attr = NodeAttr::Header(s, true);
        }
    }

    pub fn unlink_column(&mut self) {
        let up_ref = self.up();
        let up = self.up.upgrade().unwrap();
        let down_ref = self.down();
        let down = self.down.upgrade().unwrap();
        (*up).borrow_mut().down = down_ref;
        (*down).borrow_mut().up = up_ref;
    }

    pub fn relink_column(&mut self) {
        let up = self.up.upgrade().unwrap();
        (*up).borrow_mut().down = self.me.clone();
        let down = self.down.upgrade().unwrap();
        (*down).borrow_mut().up = self.me.clone();
    }

    pub fn unlink_row(&mut self) {
        let left_ref = self.left();
        let left = self.left.upgrade().unwrap();
        let right_ref = self.right();
        let right = self.right.upgrade().unwrap();
        (*left).borrow_mut().right = right_ref;
        (*right).borrow_mut().left = left_ref;
    }

    pub fn relink_row(&mut self) {
        let left = self.left.upgrade().unwrap();
        (*left).borrow_mut().right = self.me.clone();
        let right = self.right.upgrade().unwrap();
        (*right).borrow_mut().left = self.me.clone();
    }
}

pub fn insert_up(anchor: &OwnedNode, node_ref: &WeakNode) {
    let node = node_ref.upgrade().unwrap();
    {
        let mut node = (*node).borrow_mut();
        node.down = Rc::downgrade(anchor);
        node.up = anchor.borrow().up();
    }
    {
        let mut slf = (**anchor).borrow_mut();
        slf.up = node_ref.clone();
    }
    {
        let node = (*node).borrow_mut();
        let up = node.up.upgrade().unwrap();
        (*up).borrow_mut().down = node_ref.clone();
    }
}

pub fn insert_left(anchor: &OwnedNode, node_ref: &WeakNode) {
    let node = node_ref.upgrade().unwrap();
    {
        let mut node = (*node).borrow_mut();
        node.right = Rc::downgrade(anchor);
        node.left = anchor.borrow().left();
    }
    {
        let mut slf = (**anchor).borrow_mut();
        slf.left = node_ref.clone();
    }
    {
        let node = (*node).borrow_mut();
        let left = node.left.upgrade().unwrap();
        (*left).borrow_mut().right = node_ref.clone();
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.me, &other.me)
    }
}

#[derive(Debug)]
pub struct Matrix {
    root: OwnedNode,
    columns: Vec<OwnedNode>,
    rows: Vec<Vec<OwnedNode>>,
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            root: Node::new_root(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows.len()
    }

    #[inline]
    pub fn columns(&self) -> usize {
        self.columns.len()
    }

    #[inline(always)]
    fn add_row(&mut self) {
        self.rows.push(Vec::new());
    }

    #[inline(always)]
    fn add_column(&mut self) {
        let col = Node::new_header(self.columns.len());
        let col_ref = Rc::downgrade(&col);
        self.columns.push(col);
        insert_left(&self.root, &col_ref);
    }

    #[inline(always)]
    fn alloc_row(&mut self, idx: Idx) {
        while self.rows.len() <= idx {
            self.add_row();
        }
    }

    #[inline(always)]
    fn alloc_column(&mut self, idx: Idx) {
        while self.columns.len() <= idx {
            self.add_column();
        }
    }

    #[inline(always)]
    pub fn set_optional(&mut self, col: Idx) {
        self.alloc_column(col);
        self.columns[col].borrow_mut().set_optional();
    }

    pub fn set(&mut self, row_idx: Idx, col_idx: Idx) {
        self.alloc_column(col_idx);
        self.alloc_row(row_idx);

        let col = &mut self.columns[col_idx];
        let row = &mut self.rows[row_idx];

        let node = Node::new_cell(&Rc::downgrade(col), row_idx);
        let node_ref = Rc::downgrade(&node);

        if !row.is_empty() {
            for n in row.iter() {
                let h = (*n).borrow().header().upgrade().unwrap();
                if (*h).borrow().idx() == col_idx {
                    return;
                }
            }
            insert_left(&row[0], &node_ref);
        }
        insert_up(col, &node_ref);
        col.borrow_mut().inc_size();
        row.push(node);
    }

    pub fn solve(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut m = Matrix::new();

        //     0   1   2
        //   ┌───┬───┬───┐
        // 0 | 1 | 0 | 1 |
        //   ├───┼───┼───┤
        // 1 | 0 | 1 | 1 |
        //   ├───┼───┼───┤
        // 2 | 0 | 1 | 0 |
        //   ├───┼───┼───┤
        // 3 | 1 | 1 | 0 |
        //   └───┴───┴───┘

        m.set(0, 0);
        m.set(0, 2);
        m.set(1, 1);
        m.set(1, 2);
        m.set(2, 1);
        m.set(3, 0);
        m.set(3, 1);

        assert_eq!(m.columns(), 3);
        assert_eq!(m.rows(), 4);
    }
}
