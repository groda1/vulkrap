#[cfg(test)]
mod tests {
    use cgmath::{Deg, Matrix4, Point3, Quaternion, Rad, Rotation3, Vector3, Zero};
    

    #[test]
    fn test() {
        let vertex: Point3<f32> = Point3::new(1.0, 0.0, 0.0);

        let _trans: Matrix4<f32> = Matrix4::from_translation(Vector3::new(10.0, 0.0, 0.0));

        let mut quat: Quaternion<f32> = Quaternion::from_angle_y(Deg(30.0));

        let quat2 = Quaternion::from_angle_x(Deg(30.0 as f32));

        quat = quat * quat2;

        Matrix4::from(quat2);

        let trans: Matrix4<f32> = Matrix4::from_translation(Vector3::new(10.0, 0.0, 0.0));
        let rot: Matrix4<f32> = Matrix4::from_angle_y(Deg(90.0));

        println!("vec {:?}", trans * rot * vertex.to_homogeneous());
    }
}
