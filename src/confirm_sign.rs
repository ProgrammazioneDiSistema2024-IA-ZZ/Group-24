// confirm_sign.rs

#[derive(Default)]
pub struct HorizontalLineTracker {
    start_x: Option<f64>,
    start_y: Option<f64>,
    end_x: Option<f64>,
    end_y: Option<f64>,
    is_horizontal: bool,
}

impl HorizontalLineTracker {
    // Crea una nuova istanza
    pub fn new() -> Self {
        HorizontalLineTracker {
            start_x: None,
            start_y: None,
            end_x: None,
            end_y: None,
            is_horizontal: false,
        }
    }

    // Resetta il tracker per iniziare un nuovo tracciamento
    pub fn reset(&mut self) {
        *self = HorizontalLineTracker::new();
    }

    // Aggiorna il tracker con una nuova posizione del mouse
    pub fn update(&mut self, x: f64, y: f64) {
        if self.start_x.is_none() || self.start_y.is_none() {
            // Imposta il punto iniziale
            self.start_x = Some(x);
            self.start_y = Some(y);
        }

        // Aggiorna il punto finale
        self.end_x = Some(x);
        self.end_y = Some(y);

        // Verifica se il movimento è principalmente orizzontale
        if let (Some(start_x), Some(start_y), Some(end_x), Some(end_y)) = 
            (self.start_x, self.start_y, self.end_x, self.end_y) {
            let horizontal_distance = (end_x - start_x).abs();
            let vertical_distance = (end_y - start_y).abs();
            
            self.is_horizontal = horizontal_distance > 100.0 && vertical_distance < 50.0;
        }
    }

    // Controlla se il movimento soddisfa i criteri per essere una linea orizzontale
    pub fn is_valid_horizontal(&self) -> bool {
        self.is_horizontal
    }
}