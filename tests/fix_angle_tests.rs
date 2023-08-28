use std::f64::consts::PI;
use i_float:: fix_angle::FixAngle;

#[test]
fn test_0() {
    let mut angle = -PI;
    while angle < PI {
        let fix_angle = FixAngle::new_from_radians_f64(angle);

        let fix_sin = fix_angle.sin();

        let sin0 = fix_sin.double();
        let sin1 = angle.sin();
        
        let d_sin = sin0 - sin1;

        assert_eq!(d_sin.abs() < 0.01, true);

        angle += 0.001;
    }
}

#[test]
fn test_1() {
    let mut angle = -PI;
    while angle < PI {
        let fix_angle = FixAngle::new_from_radians_f64(angle);

        let fix_cos = fix_angle.cos();

        let cos0 = fix_cos.double();
        let cos1 = angle.cos();
        
        let d_cos = cos0 - cos1;

        assert_eq!(d_cos.abs() < 0.01, true);

        angle += 0.001;
    }
}


#[test]
fn test_2() {
    let mut angle = -PI;
    while angle < PI {
        let fix_angle = FixAngle::new_from_radians_f64(angle);

        let rotator = fix_angle.rotator();
        let sc0 = (rotator.x.double(), rotator.y.double());
        let sc1 = angle.sin_cos();
        
        let d_sin = sc0.0 - sc1.0;
        let d_cos = sc0.1 - sc1.1;

        assert_eq!(d_sin.abs() < 0.01, true);
        assert_eq!(d_cos.abs() < 0.01, true);

        angle += 0.001;
    }
}