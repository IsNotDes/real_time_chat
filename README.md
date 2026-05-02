# real_time_chat

Un serveur de chat temps réel écrit en Rust, avec un frontend léger en JavaScript vanilla.

Le backend gère les connexions WebSocket via **Tokio** et diffuse les messages à tous les clients connectés en temps réel.

---

## Fonctionnalités

- Communication temps réel via **WebSocket**
- Backend Rust asynchrone (Tokio)
- Broadcast des messages à tous les clients connectés
- Frontend minimaliste HTML/JS, sans dépendances
- Architecture simple et lisible, idéale pour comprendre le modèle async Rust

---

## Architecture

```
┌─────────────────────────────────────┐
│         Frontend (HTML + JS)        │
│     WebSocket client natif          │
└────────────────┬────────────────────┘
                 │ ws://localhost:PORT
┌────────────────▼────────────────────┐
│         Serveur Rust (Tokio)        │
│  Accepte les connexions entrantes   │
│  Spawn d'une tâche async par client │
│  Broadcast via canal partagé        │
└─────────────────────────────────────┘
```

---

## Installation et lancement

```bash
git clone https://github.com/IsNotDes/real_time_chat
cd real_time_chat

# Lancer le serveur Rust
cargo run

# Ouvrir frontend/index.html dans votre navigateur
```

---

## Stack technique

| Composant  | Technologie                           |
|------------|---------------------------------------|
| Serveur    | Rust + Tokio (async runtime)          |
| WebSocket  | Bibliothèque WebSocket async Rust     |
| Frontend   | HTML + JavaScript vanilla             |

---

## Commandes de développement

```bash
cargo build    # Compilation
cargo run      # Lancer le serveur
cargo fmt      # Formatter le code
cargo clippy   # Linter
```

---

## Ce que j'ai appris

Ce projet m'a permis d'explorer le modèle de concurrence asynchrone de Rust avec Tokio : gestion de multiples connexions WebSocket en parallèle via `tokio::spawn`, partage d'état entre tâches async, et diffusion de messages (broadcast) sans blocage.
