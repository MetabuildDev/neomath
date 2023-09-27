use geo::{CheckedBooleanOps, MapCoords, OpType};
use neo_float::NeoFloat;

pub trait NeoGeoBoolops<F: NeoFloat> {
    fn neo_boolop(&self, other: &Self, op: OpType) -> Option<geo::MultiPolygon<F>>;

    fn neo_union(&self, other: &Self) -> Option<geo::MultiPolygon<F>> {
        self.neo_boolop(other, OpType::Union)
    }
    fn neo_intersection(&self, other: &Self) -> Option<geo::MultiPolygon<F>> {
        self.neo_boolop(other, OpType::Intersection)
    }
    fn neo_difference(&self, other: &Self) -> Option<geo::MultiPolygon<F>> {
        self.neo_boolop(other, OpType::Difference)
    }
}

impl<F, Boolable, BoolableMappedF64, BoolableMappedF> NeoGeoBoolops<F> for Boolable
where
    F: NeoFloat,
    BoolableMappedF64: CheckedBooleanOps<Scalar = f64> + MapCoords<f64, F, Output = Boolable>,
    BoolableMappedF: CheckedBooleanOps<Scalar = F>,
    Boolable: CheckedBooleanOps<Scalar = F>
        + MapCoords<F, f64, Output = BoolableMappedF64>
        + MapCoords<F, F, Output = BoolableMappedF>,
{
    fn neo_boolop(&self, other: &Self, op: OpType) -> Option<geo::MultiPolygon<F>> {
        let res = self
            .checked_boolean_op(other, op)
            .or_else(|_| {
                let s = self.map_coords(coord_upcast);
                let o = other.map_coords(coord_upcast);
                s.checked_boolean_op(&o, op)
                    .map(|res| res.map_coords(coord_downcast))
            })
            .ok();
        #[cfg(feature = "random-retry")]
        let res = {
            use rand::Rng;
            let offset_range = || -0.000001..=0.000001;
            let random_coord = || {
                let mut rng = rand::thread_rng();
                geo::Coord::<F> {
                    x: F::from_raw_f64(rng.gen_range(offset_range())),
                    y: F::from_raw_f64(rng.gen_range(offset_range())),
                }
            };
            let mut max_retries = 10;
            let mut res = res;
            while res.is_none() && max_retries > 0 {
                res = res.or_else(|| {
                    let s = self.map_coords(|c| c + random_coord());
                    let o = other.map_coords(|c| c + random_coord());
                    s.checked_boolean_op(&o, op).ok()
                });
                max_retries -= 1;
            }
            res
        };
        res
    }
}

fn coord_upcast<F: NeoFloat>(c: geo::Coord<F>) -> geo::Coord<f64> {
    geo::Coord {
        x: c.x.to_f64().unwrap(),
        y: c.y.to_f64().unwrap(),
    }
}

fn coord_downcast<F: NeoFloat>(c: geo::Coord<f64>) -> geo::Coord<F> {
    geo::Coord {
        x: F::from_raw_f64(c.x),
        y: F::from_raw_f64(c.y),
    }
}

#[test]
fn f32_version_compiles() {
    type T = f32;
    let poly = geo::Polygon::<T>::new(geo::LineString::<T>::new(vec![]), vec![]);
    _ = poly.checked_union(&poly);

    let multi_poly = geo::MultiPolygon::<T>::new(vec![]);
    _ = multi_poly.checked_union(&multi_poly);
}

#[test]
fn f64_version_compiles() {
    type T = f64;
    let poly = geo::Polygon::<T>::new(geo::LineString::<T>::new(vec![]), vec![]);
    _ = poly.checked_union(&poly);

    let multi_poly = geo::MultiPolygon::<T>::new(vec![]);
    _ = multi_poly.checked_union(&multi_poly);
}
