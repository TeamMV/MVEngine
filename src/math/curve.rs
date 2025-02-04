use mvutils::utils;
use mvutils::utils::Factorial;

const BEZIER_PRECISION: usize = 100;

#[derive(Clone)]
pub struct SimpleBezierCurve {
    pub grade: usize,
    compiled: [f64; BEZIER_PRECISION],
}

impl SimpleBezierCurve {
    pub fn new(points: &[f64]) -> Self {
        let grade = points.len();

        let mut this = Self {
            grade,
            compiled: [0.0; BEZIER_PRECISION],
        };

        for i in 0..BEZIER_PRECISION {
            let pos = 1.0 / BEZIER_PRECISION as f64 * i as f64;
            let res = Self::run_bezier_once(pos, grade, points);
            this.compiled[i] = res;
        }

        this
    }

    fn run_bezier_once(pos: f64, n: usize, points: &[f64]) -> f64 {
        let n64 = n as f64;
        let mut res = 0.0;
        for (idx, i) in (0..n).enumerate() {
            let i64 = i as f64;
            res += ((n.fact() as f64 / (i.fact() * (n - i).fact()) as f64)
                * pos.powf(i64)
                * (1.0 - pos).powf(n64 - i64))
                * points[idx]
        }
        res
    }

    pub fn get(&self, pos: f64) -> f64 {
        if pos < 0.0 || pos > 1.0 {
            return 0.0;
        }
        let x = BEZIER_PRECISION as f64 * pos;
        let idx = x.floor() as usize;

        let p = x - idx as f64;

        if idx == BEZIER_PRECISION - 1 {
            return self.compiled[idx];
        }
        let near = self.compiled[idx];
        let far = self.compiled[idx + 1];

        utils::lerp(near, far, p)
    }
}
