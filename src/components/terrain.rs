use bevy::prelude::*;
use std::collections::HashMap;

// --- COMPOSANTS ---

#[derive(Component)]
pub struct TerrainCell {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct SolidObject {
    pub material_id: String,
    pub health: f32,
    pub max_health: f32,
    pub mergeable: bool,
    // Stocke le pattern de voisinage pour déterminer quelle texture utiliser
    pub neighbors_pattern: u8,
}

#[derive(Component)]
pub struct WorldEntityItem {
    pub entity_id: String,
    pub quantity: i32,
}

#[derive(Component)]
pub struct HoverState {
    pub hovered: bool,
}

// --- RESSOURCES ---

#[derive(Resource)]
pub struct WorldMaterials {
    pub materials: HashMap<String, MaterialDefinition>,
}

#[derive(Resource)]
pub struct WorldEntities {
    pub entities: HashMap<String, EntityDefinition>,
}

// --- ÉVÉNEMENTS ---

#[derive(Event)]
pub struct UpdateTerrainEvent {
    pub region: Option<(Vec2, Vec2)>,
}

// --- DÉFINITIONS ---

#[derive(Clone)]
pub struct MaterialDefinition {
    pub name: String,
    pub strength: f32,
    pub drop_entity_id: String,
    pub drop_count_min: i32,
    pub drop_count_max: i32,
    pub can_be_merged: bool,
    pub rarity: f32, // Rareté du matériau (0.0 = commun, 1.0 = très rare)
    pub plain_texture: Handle<Image>,
    pub side_texture: Handle<Image>,
    pub inter_corner_texture: Handle<Image>,
    pub outer_corner_texture: Handle<Image>,
}

#[derive(Clone)]
pub struct EntityDefinition {
    pub name: String,
    pub icon: Handle<Image>,
    pub max_stack: i32,
}

// --- CONSTANTES ---

// Couleur du sol martien
pub const MARS_GROUND_COLOR: Color = Color::srgb(
    192.0 / 255.0,
    122.0 / 255.0,
    91.0 / 255.0,
);

// Directions pour le voisinage (bits 0-7 pour les 8 directions)
// Format: Bit 0 = Droite, 1 = Haut-Droite, 2 = Haut, 3 = Haut-Gauche,
//         4 = Gauche, 5 = Bas-Gauche, 6 = Bas, 7 = Bas-Droite
pub const NEIGHBOR_RIGHT: u8 = 0b00000001;
pub const NEIGHBOR_TOP_RIGHT: u8 = 0b00000010;
pub const NEIGHBOR_TOP: u8 = 0b00000100;
pub const NEIGHBOR_TOP_LEFT: u8 = 0b00001000;
pub const NEIGHBOR_LEFT: u8 = 0b00010000;
pub const NEIGHBOR_BOTTOM_LEFT: u8 = 0b00100000;
pub const NEIGHBOR_BOTTOM: u8 = 0b01000000;
pub const NEIGHBOR_BOTTOM_RIGHT: u8 = 0b10000000;

// Implémentation par défaut pour les ressources
impl Default for WorldMaterials {
    fn default() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }
}

impl Default for WorldEntities {
    fn default() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
}

// --- MÉTHODES UTILITAIRES ---

impl SolidObject {
    // Détermine quelle texture utiliser en fonction du motif de voisinage
    pub fn get_texture(&self, world_materials: &WorldMaterials) -> Option<Handle<Image>> {
        if let Some(material_def) = world_materials.materials.get(&self.material_id) {
            if !self.mergeable {
                return Some(material_def.plain_texture.clone());
            }

            // Logique pour choisir la texture en fonction des voisins
            if self.is_outer_corner() {
                Some(material_def.outer_corner_texture.clone())
            } else if self.is_inner_corner() {
                Some(material_def.inter_corner_texture.clone())
            } else if self.is_side() {
                Some(material_def.side_texture.clone())
            } else {
                Some(material_def.plain_texture.clone())
            }
        } else {
            None
        }
    }

    // Vérifie si c'est un coin extérieur (un seul voisin diagonal)
    pub fn is_outer_corner(&self) -> bool {
        let pattern = self.neighbors_pattern;
        // Vérifie les cas de coins extérieurs
        (pattern == NEIGHBOR_TOP_RIGHT) ||
            (pattern == NEIGHBOR_TOP_LEFT) ||
            (pattern == NEIGHBOR_BOTTOM_LEFT) ||
            (pattern == NEIGHBOR_BOTTOM_RIGHT)
    }

    // Vérifie si c'est un coin intérieur (trois voisins mais pas en diagonale)
    pub fn is_inner_corner(&self) -> bool {
        let pattern = self.neighbors_pattern;
        // Vérifie les cas de coins intérieurs
        (pattern & (NEIGHBOR_RIGHT | NEIGHBOR_TOP) == (NEIGHBOR_RIGHT | NEIGHBOR_TOP) &&
            pattern & NEIGHBOR_TOP_RIGHT == 0) ||
            (pattern & (NEIGHBOR_TOP | NEIGHBOR_LEFT) == (NEIGHBOR_TOP | NEIGHBOR_LEFT) &&
                pattern & NEIGHBOR_TOP_LEFT == 0) ||
            (pattern & (NEIGHBOR_LEFT | NEIGHBOR_BOTTOM) == (NEIGHBOR_LEFT | NEIGHBOR_BOTTOM) &&
                pattern & NEIGHBOR_BOTTOM_LEFT == 0) ||
            (pattern & (NEIGHBOR_BOTTOM | NEIGHBOR_RIGHT) == (NEIGHBOR_BOTTOM | NEIGHBOR_RIGHT) &&
                pattern & NEIGHBOR_BOTTOM_RIGHT == 0)
    }

    // Vérifie si c'est un côté (un seul voisin orthogonal)
    pub fn is_side(&self) -> bool {
        let pattern = self.neighbors_pattern;
        // Vérifie les cas de côtés
        pattern == NEIGHBOR_RIGHT ||
            pattern == NEIGHBOR_TOP ||
            pattern == NEIGHBOR_LEFT ||
            pattern == NEIGHBOR_BOTTOM
    }
}
