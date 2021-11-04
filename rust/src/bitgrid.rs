use hibitset::BitSet;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct BitGrid {
    width: u32,
    height: u32,
    grid: BitSet,
}

impl BitGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            grid: BitSet::with_capacity(width * height),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> u32 {
        self.width * self.height
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        self.grid.contains(self.as_index(x, y))
    }

    pub fn add(&mut self, x: u32, y: u32) -> bool {
        self.grid.add(self.as_index(x, y))
    }

    pub fn remove(&mut self, x: u32, y: u32) -> bool {
        self.grid.remove(self.as_index(x, y))
    }

    pub fn clear(&mut self) {
        self.grid.clear()
    }

    fn as_index(&self, x: u32, y: u32) -> u32 {
        assert!(
            x < self.width && y < self.height,
            "coordinates out of range"
        );
        y * self.width + x
    }
}
