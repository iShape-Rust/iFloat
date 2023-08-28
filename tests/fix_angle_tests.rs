use std::f64::consts::PI;
use i_float::{ fix_angle::FixAngle, fix_float::FixFloat};

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

#[test]
fn test_3() {
    let mut angle: f64 = 0.0;
    let to_rad = PI / 180.0;
    while angle < 360.0 {
        let fix_angle = FixAngle::new_from_degrees_f64(angle);

        let rotator = fix_angle.rotator();
        let sc0 = (rotator.x.double(), rotator.y.double());
        let sc1 = (angle * to_rad).sin_cos();
        
        let d_sin = sc0.0 - sc1.0;
        let d_cos = sc0.1 - sc1.1;

        assert_eq!(d_sin.abs() < 0.01, true);
        assert_eq!(d_cos.abs() < 0.01, true);

        angle += 0.5;
    }
}

#[test]
fn test_4() {
    let mut angle: f64 = 0.0;
    while angle < 360.0 {
        let fix_float = FixFloat::new_f64(angle);
        
        let fix_angle0 = FixAngle::new_from_degrees_f64(angle);
        let fix_angle1 = FixAngle::new_from_degrees_fix(fix_float);

        assert_eq!(fix_angle0.value(), fix_angle1.value());

        angle += 0.5;
    }
}

#[test]
fn test_5() {
    let mut radians: f64 = -PI;
    while radians < PI - 0.02 {
        let fix_float = FixFloat::new_f64(radians);
        
        let fix_angle0 = FixAngle::new_from_radians_f64(radians);
        let fix_angle1 = FixAngle::new_from_radians_fix(fix_float);

        let d0 = (fix_angle0.value() - fix_angle1.value()).abs();

        assert_eq!(d0 < 10, true);

        radians += 0.01;
    }
}