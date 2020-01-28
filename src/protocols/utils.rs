use core::marker::PhantomData;
use core::ops::Range;

#[derive(Debug, Clone)]
pub struct PulseWidthRange<T> {
    r: [Range<u32>; 4],
    pd: PhantomData<T>,
}

impl<T> PulseWidthRange<T>
where
    T: Default + From<usize>,
{
    pub fn new(vals: &[(u32, u32); 4]) -> Self {
        PulseWidthRange {
            r: [
                pulserange(vals[0].0, vals[0].1),
                pulserange(vals[1].0, vals[1].1),
                pulserange(vals[2].0, vals[2].1),
                pulserange(vals[3].0, vals[3].1),
            ],
            pd: PhantomData,
        }
    }

    pub fn pulsewidth(&self, pulsewidth: u32) -> T {
        self.r
            .iter()
            .position(|r| r.contains(&pulsewidth))
            .map(T::from)
            .unwrap_or_default()
    }
}

const fn pulserange(units: u32, tolerance: u32) -> Range<u32> {
    let tol = (units * tolerance) / 100;
    (units - tol..units + tol)
}


pub fn unsigned_abs_sat_sub(t0: u32, t1: u32) -> u32 {
    use core::{u32, i32};

    let (dt, overflow) = t1.overflowing_sub(t0);

    if !overflow {
        dt
    } else {
        (dt as i32)
            .checked_abs()
            .map(|n| n as u32)
            .unwrap_or(i32::MAX as u32)
    }
}


#[cfg(test)]
mod test {
    use crate::protocols::utils::unsigned_abs_sat_sub;

    #[test]
    fn sampletime() {
        use std::{u16, u32, i32};

        println!("i32::MAX: {}", i32::MAX);

        let tests = [
            // res = t1 - t0
            (0,  0,  0),
            (10, 10, 20),   // Wraps
            // Counter wraps around between t0 and t1
            (10, 9, u32::MAX), // Wraps

            (u32::MAX/2, (i32::MAX - 1) as u32, u32::MAX),
            (i32::MAX as u32, (i32::MAX) as u32, u32::MAX),

            (0, u32::MAX, u32::MAX),
            // Ladida
            (u32::MAX, u32::MAX, 0)
            //(u32::MAX-10, 99, 110),
            //(u32::MAX, (u32::MAX/2-1), (u32::MAX/2)),
            //(u32::MAX-1000, (u32::MAX/2-1001), (u32::MAX/2)),
        ];

        for &(res, t1, t0) in &tests {
            println!("{} = abs({} - {})", res, t1, t0);
            let r = unsigned_abs_sat_sub(t0, t1);
            assert_eq!(r, res);
        }
    }
}

