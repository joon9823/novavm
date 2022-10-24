use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SizeDelta {
    pub amount: usize,
    pub is_decrease: bool,
}

#[allow(dead_code)]
impl SizeDelta {
    pub fn zero() -> Self {
        Self {
            amount: 0,
            is_decrease: false,
        }
    }

    pub fn new(old: usize, new: usize) -> Self {
        Self {
            amount: new.abs_diff(old),
            is_decrease: new < old,
        }
    }

    pub fn increasing(amount: usize) -> Self {
        Self {
            amount,
            is_decrease: false,
        }
    }

    pub fn decreasing(amount: usize) -> Self {
        Self {
            amount,
            is_decrease: true,
        }
    }

    // TODO: it panics if overflow. How do we handle?
    pub fn merge(&mut self, delta: SizeDelta) {
        if self.is_decrease == delta.is_decrease {
            self.amount += delta.amount;
        } else {
            if self.amount < delta.amount {
                self.is_decrease = !self.is_decrease;
            }
            self.amount = self.amount.abs_diff(delta.amount);
        }
    }
}

#[cfg(test)]
mod test {
    use super::SizeDelta;

    #[test]
    fn test_size_delta() {
        let mut z = SizeDelta::zero();
        z.merge(SizeDelta::new(10, 0));
        assert_eq!(z.amount, 10);
        assert_eq!(z.is_decrease, true);

        //a is 1
        let mut a = SizeDelta::new(2, 3);
        assert_eq!(a.amount, 1);
        assert_eq!(a.is_decrease, false);

        // b is -5
        let b = SizeDelta::new(10, 5);
        assert_eq!(b.amount, 5);
        assert_eq!(b.is_decrease, true);

        // now a is 3
        a.merge(SizeDelta {
            amount: 2,
            is_decrease: false,
        });
        assert_eq!(a.amount, 3);
        assert_eq!(a.is_decrease, false);

        // now a is -2
        a.merge(b);
        assert_eq!(a.amount, 2);
        assert_eq!(a.is_decrease, true);

        // now a is -7
        a.merge(SizeDelta {
            amount: 5,
            is_decrease: true,
        });
        assert_eq!(a.amount, 7);
        assert_eq!(a.is_decrease, true);
    }
}

impl fmt::Display for SizeDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            if self.is_decrease { "-" } else { "+" },
            self.amount
        )
    }
}
