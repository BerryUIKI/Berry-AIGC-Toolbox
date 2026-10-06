# Sauvegarde d'instantanés Cloud & Miroir multimédia

Omera intègre un moteur de sauvegarde cloud et de synchronisation différentielle delta (`src-tauri/src/cloud_backup.rs` et `src-tauri/src/cloud_sync.rs`) qui permet des sauvegardes automatisées de base de données et la mise en miroir incrémentielle des fichiers médias distants, sans recourir à des outils tiers.

---

## 1. Fournisseurs de stockage pris en charge

Configurez vos points de terminaison distants dans **Préférences > Sauvegarde Cloud** :

| Fournisseur | Protocoles & Points de terminaison pris en charge | Remarques |
| :--- | :--- | :--- |
| **AWS S3 / Compatible** | AWS S3, Cloudflare R2, MinIO, Backblaze B2, Wasabi | Authentification native en Rust avec AWS Signature Version 4 (SigV4) signée en HMAC-SHA256. |
| **WebDAV** | Nextcloud, ownCloud, Synology DiskStation, QNAP NAS | Authentification HTTP Basic standard sur HTTPS. |
| **Chemin local / Réseau** | Disque local, SSD externe USB, partages réseau SMB / NFS | Entrées/sorties directes haute performance sur le système de fichiers sans surcharge protocolaire réseau. |

---

## 2. Sauvegarde d'instantanés SQLite à chaud (`cloud_backup_create_snapshot`)

Omera effectue la sauvegarde de votre base de données à l'aide de l'instruction native SQLite `VACUUM INTO` :

```mermaid
sequenceDiagram
    participant UI as Interface Omera Studio
    participant Rust as Backend (cloud_backup.rs)
    participant DB as SQLite WAL (omera.db)
    participant Remote as Stockage Cloud (S3/WebDAV)

    UI->>Rust: Demande de création d'instantané
    Rust->>DB: VACUUM INTO temp_snapshot.db (Sans verrouillage)
    DB-->>Rust: Copie cohérente de la base à l'instant T
    Rust->>Rust: Empaquetage en ZIP avec manifest.json
    Rust->>Remote: Téléversement en flux continu (SigV4 / WebDAV PUT)
    Remote-->>Rust: Téléversement confirmé (200 OK)
    Rust-->>UI: Instantané créé avec succès
```

### Garanties des instantanés :
- **Sans interruption (Non-Locking)** : Utilise l'API de vacuum en ligne de SQLite. Vous pouvez continuer à parcourir, noter et générer des images sans aucune interruption de service.
- **Sécurité de restauration (Rollback)** : Lors de la restauration d'un instantané distant, Omera génère automatiquement une copie locale de secours (`omera.db.rollback`) avant d'appliquer la sauvegarde distante, prévenant toute coupure réseau ou corruption de téléchargement.

---

## 3. Miroir multimédia incrémentiel & Synchronisation Delta (`cloud_sync.rs`)

Tandis que les instantanés sécurisent votre base de données, la **Synchronisation Delta** offre une mise en miroir unidirectionnelle par téléversement incrémentiel des fichiers physiques d'images et de vidéos depuis votre bibliothèque locale vers un stockage distant ou réseau.

### Fonctionnalités de synchronisation (Implémentées) :
- **Stratégies de détection des modifications** :
  - *Empreinte rapide* : Compare la taille du fichier local et celle du fichier distant via les métadonnées HTTP HEAD (la plus rapide, idéale pour les connexions modérées).
  - *Somme de contrôle stricte* : Calcule des sommes SHA-256 locales pour vérifier la conformité avec les en-têtes distants (`x-amz-meta-sha256` sur S3) octet par octet.
- **Limiteur de bande passante par seau de jetons** : Fixez un plafond de vitesse d'envoi (Ko/s) afin que la synchronisation d'arrière-plan ne sature pas la bande passante de votre studio.
- **Concurrence des workers** : Configurez le nombre de threads d'envoi simultanés (de 1 à 16 threads, 4 par défaut).
- **Mode simulation (Dry-Run)** : Simule l'exécution de la synchronisation et indique les fichiers à téléverser et à ignorer sans modifier le stockage distant.
- **Suivi de progression en temps réel** : Émet des événements réguliers indiquant les octets transférés, le débit, le pourcentage achevé et le temps estimé restant, avec annulation atomique coopérative.

### Limites actuelles et fonctionnalités prévues :
- **Téléversement unidirectionnel uniquement** : Le chemin de synchronisation actuel pousse les fichiers locaux indexés vers la cible distante. Le téléchargement/rapatriement de fichiers distants vers le stockage local n'est pas implémenté.
- **Aucune synchronisation des suppressions** : Supprimer un fichier localement ne supprime pas le fichier distant ; le stockage distant conserve les médias.
- **Aucune logique delta par ETag** : La détection s'appuie sur la taille et les en-têtes SHA-256 ; les ETag HTTP du fournisseur ne sont pas utilisés pour invalider le cache.
- **Synchronisation bidirectionnelle planifiée** : Une synchronisation bidirectionnelle complète avec détection des modifications distantes, réconciliation descendante et gestion des conflits est planifiée pour de futures versions.
