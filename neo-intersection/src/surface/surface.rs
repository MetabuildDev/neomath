use geo::{MapCoords, SpadeBoolops};
use glam::Vec3;
use neo_surface::surface::def::NeoSurface;

use crate::line_intersection_parts::Line3DIntersectionParts;
use crate::surface::coord_sys::SurfaceCoordSys3DIntersection;
use crate::trait_def::NeoIntersectable;

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceSurface3DIntersection {
    None,
    Point(Vec3),
    Parts(Vec<Line3DIntersectionParts>),
    Surface(NeoSurface),
    MultiSurface(Vec<NeoSurface>),
}

impl NeoIntersectable for NeoSurface {
    type Output = SurfaceSurface3DIntersection;

    fn intersection(&self, rhs: &Self) -> Self::Output {
        let inter = self.intersection(&rhs.coordinate_system);
        match inter {
            SurfaceCoordSys3DIntersection::None => SurfaceSurface3DIntersection::None,
            // the next two imply that the intersection is happening because of coordinate systems
            // which are not parallel so we can just pass them through
            SurfaceCoordSys3DIntersection::Point(p) => SurfaceSurface3DIntersection::Point(p),
            SurfaceCoordSys3DIntersection::Parts(ps) => SurfaceSurface3DIntersection::Parts(ps),
            SurfaceCoordSys3DIntersection::Surface(_) => {
                let intersection = surface_intersection_case_analysis(self, rhs);
                match intersection.len() {
                    0 => SurfaceSurface3DIntersection::None,
                    1 => SurfaceSurface3DIntersection::Surface(intersection[0].clone()),
                    _ => SurfaceSurface3DIntersection::MultiSurface(intersection),
                }
            }
        }
    }
}

fn surface_intersection_case_analysis(surface: &NeoSurface, rhs: &NeoSurface) -> Vec<NeoSurface> {
    // we are working in the coordinate system of the `surface` argument
    let origin_2d_diff = surface.shape_origin - rhs.shape_origin;
    let rhs_translated_shape = rhs.shape.map_coords(|c| c + origin_2d_diff);
    let intersection = SpadeBoolops::intersection(&surface.shape, &rhs_translated_shape).unwrap();

    // if it fails, we just return no intersection at all which might be wrong
    intersection
        .into_iter()
        .map(|p| NeoSurface {
            shape: p,
            ..surface.clone()
        })
        .collect::<Vec<_>>()
}
