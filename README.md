[![wakatime](https://wakatime.com/badge/user/018da7b9-5ddd-4615-a805-e871e840191c/project/b74fcbcf-2616-4691-b7f8-08150f738a46.svg)](https://wakatime.com/badge/user/018da7b9-5ddd-4615-a805-e871e840191c/project/b74fcbcf-2616-4691-b7f8-08150f738a46)

# Mars Mechanica

## Description

Mars Mechanica est un projet scolaire réalisé en Rust qui a pour but de simuler un essaim de robots sur une planète
Mars.
Les robots sont capables de se déplacer, de communiquer entre eux et de récolter des ressources. Ils doivent coopérer
pour survivre et accomplir des objectifs.

Le projet a été développé avec le moteur de jeu Bevy, un framework moderne et orienté entité-composant-système (ECS)
pour Rust.

## Fonctionnalités

- Génération procédurale de terrain martien avec différents matériaux (roches, basalte, olivine, cristaux rouges). Les
  cristaux rouges représentent une ressource d'énergie (comparable à de l'uranium) et sont essentiels pour le
  fonctionnement des robots.
- Système de chunks pour une meilleure performance
- Deux types de robots avec des comportements distincts :
    - Robot explorateur : découvre le terrain et partage ses découvertes
    - Robot mineur : exploite les ressources découvertes (notamment les cristaux rouges)
- Interface de débogage pour visualiser et manipuler l'environnement
- Système de seed pour reproduire des mondes identiques

## Simplifications apportées au sujet

- Le robot explorateur partage sa base de données avec les autres robots en temps réel. Il n'a pas besoin de retourner à
  la base pour partager ses découvertes.
- Les robots peuvent voir les cellules vides jusqu'à une distance de 8 blocs, mais ne peuvent détecter les objets
  solides qu'à une distance de 1 bloc, simulant ainsi des limitations de perception plus réalistes.

## Installation

### Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (édition 2024)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### Compilation et exécution

```bash
# Cloner le dépôt
git clone https://github.com/votre-utilisateur/mars-mechanica.git
cd mars-mechanica

# Compiler et exécuter en mode développement
cargo run

# Compiler en mode release pour de meilleures performances
cargo build --release
./target/release/mars-mechanica
```

## Utilisation

### Contrôles

- **Souris droite + glisser** : Navigation de la caméra
- **Molette** : Zoom in/out
- **Clic gauche** : Interaction avec les blocs (selon le mode sélectionné dans la toolbox)

### Paramètres de démarrage

- `--skip-splash` : Ignorer l'animation d'introduction (accélère **grandement** le chargement du jeu en compilation dev)

## Configuration

Les valeurs principales à ajuster sont dans le fichier `src/components/terrain.rs` :

- `CHUNK_SIZE` : Taille des chunks (en blocs)
- `MAP_SIZE` : Nombre de chunks dans chaque direction
- `CELL_SIZE` : Taille de chaque bloc en pixels

## Problèmes connus

- Le système de chunk casse le terrain en le coupant aux extrémités de la zone actualisée (lors d'une action).
- L'algorithme de placement des sprites, gérant la détection des blocs voisins et le masquage des bordures n'est pas
  terminé. Le modèle actuel est très "manuel" et peu efficace. Certaines jointures entre certains patterns ne sont pas
  correctes, voir manquantes.
- Il faut parfois double cliquer pour détruire un bloc.
- La destruction d'un bloc n'efface pas tous les sprites de masquage.
- Les performances sont mauvaises sur des mondes dépassant les 100 x 100 blocs (même si c'est mieux depuis la mise en
  place des chunks).
- Les blocs de critaux rouges ne sont pas détectés s'ils ne sont pas placés sur un flan de montagne.
- Cliquer sur la toolbox revient à cliquer sur le terrain, ce qui peut causer des interactions non désirées.

## Wiki

Si vous êtes intéressé par le projet, vous pouvez consulter le wiki pour plus de détails.

https://github.com/SofianeLasri/mars-mechanica/wiki

## Crédits

- [Rust](https://www.rust-lang.org/)
- [Bevy](https://bevyengine.org/)
- Textures provenant du jeu [Rimworld](https://rimworldgame.com/)