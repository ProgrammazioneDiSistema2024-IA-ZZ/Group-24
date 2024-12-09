#[derive(Default, Debug)]
pub struct ScreenEdges {
    pub top: Vec<f64>,    // memorizziamo coordinate x per top
    pub right: Vec<f64>,  // memorizziamo coordinate y per right
    pub bottom: Vec<f64>, // memorizziamo coordinate x per bottom
    pub left: Vec<f64>,   // memorizziamo coordinate y per left
}

impl ScreenEdges {
    pub fn update_edges_rectangle(
        &mut self,
        x: f64,
        y: f64,
        screen_width: f64,
        screen_height: f64,
        tolerance: f64,
    ) {
        // Se il punto è sul bordo superiore (y ≈ 0)
        if (y - 0.0).abs() <= tolerance && x >= 0.0 && x <= screen_width {
            if !self.top.contains(&x) {
                self.top.push(x);
            }
        }

        // Se il punto è sul bordo destro (x ≈ screen_width)
        if (x - screen_width).abs() <= tolerance && y >= 0.0 && y <= screen_height {
            if !self.right.contains(&y) {
                self.right.push(y);
            }
        }

        // Se il punto è sul bordo inferiore (y ≈ screen_height)
        if (y - screen_height).abs() <= tolerance && x >= 0.0 && x <= screen_width {
            if !self.bottom.contains(&x) {
                self.bottom.push(x);
            }
        }

        // Se il punto è sul bordo sinistro (x ≈ 0)
        if (x - 0.0).abs() <= tolerance && y >= 0.0 && y <= screen_height {
            if !self.left.contains(&y) {
                self.left.push(y);
            }
        }
    }

    /// Verifica se è stato fatto tutto il contorno.
    /// Logica proposta:
    /// - Ordina i vettori
    /// - Controlla copertura minima dei bordi
    pub fn is_contour_complete(&self, screen_width: f64, screen_height: f64, segment_count: usize) -> bool {
        // segment_count indica in quanti segmenti dividiamo ciascun lato
        // Ad esempio, se segment_count = 10, significa che ognuno dei quattro lati
        // deve avere almeno un punto in ognuno dei 10 segmenti in cui lo dividiamo.

        // Controlliamo top e bottom (entrambi dipendono da x)
        if !self.check_coverage(&self.top, screen_width, segment_count) {
            return false;
        }

        if !self.check_coverage(&self.bottom, screen_width, segment_count) {
            return false;
        }

        // Controlliamo left e right (entrambi dipendono da y)
        if !self.check_coverage(&self.left, screen_height, segment_count) {
            return false;
        }

        if !self.check_coverage(&self.right, screen_height, segment_count) {
            return false;
        }

        true
    }

    fn check_coverage(&self, points: &Vec<f64>, length: f64, segment_count: usize) -> bool {
        if points.is_empty() {
            return false;
        }

        let mut sorted = points.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Suddividiamo il lato in segment_count parti
        // e verifichiamo che in ogni segmento ci sia almeno un punto
        let segment_length = length / (segment_count as f64);
        let mut covered_segments = vec![false; segment_count];

        for p in sorted {
            // Determiniamo in quale segmento cade p
            let seg_index = (p / segment_length).floor() as usize;
            if seg_index < segment_count {
                covered_segments[seg_index] = true;
            }
        }

        // Se tutti i segmenti sono coperti, il lato è "completo"
        covered_segments.into_iter().all(|c| c)
    }

    pub fn reset(&mut self) {
        self.top.clear();
        self.right.clear();
        self.bottom.clear();
        self.left.clear();
        println!("Bordi resettati.");
    }
}
