[![wakatime](https://wakatime.com/badge/user/018da7b9-5ddd-4615-a805-e871e840191c/project/b74fcbcf-2616-4691-b7f8-08150f738a46.svg)](https://wakatime.com/badge/user/018da7b9-5ddd-4615-a805-e871e840191c/project/b74fcbcf-2616-4691-b7f8-08150f738a46)

# Mars mechanica

## Description

Ceci est un projet scolaire réalisé en Rust qui a pour but de simuler un essaim de robots sur une planète Mars.
Les robots sont capables de se déplacer, de communiquer entre eux et de récolter des ressources. Ils doivent coopérer
pour survivre et accomplir des objectifs.

## Information

Les valeurs à "tweaker" sont dans le fichier `src/components/terrain.rs`.

## Problèmes connus

- Le système de chunk casse le terrain en le coupant aux extrémités de la zone actualisée (lors d'une action).
- L'algorithme de placement des sprites, gérant la détection des blocs voisins et le masquage des bordures n'est pas
  terminé. Le modèle actuel est très "manuel" et peu efficace. Certaines jointures entre certains patterns ne sont pas
  correctes, voir manquantes.
- Il faut parfois double cliquer pour détruire un bloc.
- La destruction d'un bloc n'efface pas tous les sprites de masquage.
- Les performances sont mauvaises sur des mondes dépassant les 100 x 100 blocs (même si c'est mieux depuis la mise en
  place des chunks).

## Crédits

- [Rust](https://www.rust-lang.org/)
- [Bevy](https://bevyengine.org/)
- Textures provenant du jeu [Rimworld](https://rimworldgame.com/)