
#[derive(Default, Debug)]
pub struct ScreenEdges {
    pub top: Vec<f64>,
    pub right: Vec<f64>,
    pub bottom: Vec<f64>,
    pub left: Vec<f64>,
}

impl ScreenEdges {
    // Metodo per aggiornare i bordi
    pub fn update_edges_rectangle(
        &mut self,
        x: f64,
        y: f64,
        screen_width: f64,
        screen_height: f64,
        tolerance: f64,
    ) {
        if (y - 0.0).abs() <= tolerance {
            if !self.top.contains(&x) && x >= 0.0 && x <= screen_width {
                self.top.push(x);
            }
        }
        if (x - screen_width).abs() <= tolerance {
            if !self.right.contains(&y) && y >= 0.0 && y <= screen_height {
                self.right.push(y);
            }
        }
        if (y - screen_height).abs() <= tolerance {
            if !self.bottom.contains(&x) && x >= 0.0 && x <= screen_width {
                self.bottom.push(x);
            }
        }
        if (x - 0.0).abs() <= tolerance {
            if !self.left.contains(&y) && y >= 0.0 && y <= screen_height {
                self.left.push(y);
            }
        }
    }

    // Metodo per verificare se tutti i bordi sono stati toccati
    pub fn all_edges_touched(&self) -> bool {
        self.top.len() > 0 && self.right.len() > 0 && self.bottom.len() > 0 && self.left.len() > 0
    }

    // Metodo per resettare i bordi
    pub fn reset(&mut self) {
        self.top.clear();
        self.right.clear();
        self.bottom.clear();
        self.left.clear();
        println!("Bordi resettati.");
    }
}

