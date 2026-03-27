<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows"/>
  <img src="https://img.shields.io/github/license/Nyx-Off/NyxIP?style=for-the-badge" alt="License"/>
  <img src="https://img.shields.io/github/v/release/Nyx-Off/NyxIP?style=for-the-badge" alt="Release"/>
</p>

<h1 align="center">⚡ NyxIP</h1>
<p align="center"><strong>Scanner IP reseau rapide et moderne, ecrit entierement en Rust.</strong></p>
<p align="center">
  <em>Alternative legere a Angry IP Scanner — un seul executable, zero dependance.</em>
</p>

---

## Apercu

NyxIP est un scanner reseau avec interface graphique native qui detecte les hotes actifs sur votre reseau local. Il fournit pour chaque hote : adresse IP, hostname, adresse MAC, fabricant, latence, ports ouverts et statut.

### Fonctionnalites

| Fonctionnalite | Description |
|---|---|
| **Ping ICMP** | Detection via Windows API (`IcmpSendEcho`) + fallback systeme |
| **Resolution MAC** | ARP via `SendARP` (Windows API) pour le reseau local |
| **Hostname** | DNS reverse lookup automatique |
| **Vendor MAC** | Base OUI integree (~200 fabricants courants) |
| **Scan de ports** | TCP connect async sur 22 ports courants (optionnel) |
| **Plages IP** | CIDR (`192.168.1.0/24`), plage (`1-254`), IP unique |
| **Export XLSX** | Fichier Excel formate avec couleurs et filtres |
| **Menu contextuel** | Clic droit : navigateur, explorateur SMB, RDP, SSH, copie |
| **Scan concurrent** | 128 taches paralleles via Tokio |
| **Detection auto** | Pre-remplit la plage IP du reseau local |

### Interface

- Theme sombre cyberpunk (fond #12121E, accents cyan/violet)
- Tableau triable par colonne avec code couleur
- Barre de progression en temps reel
- Fenetre de credits

---

## Installation

### Prerequis

- **Windows 10/11** (utilise les API Windows natives)
- **Rust 1.70+** (pour compiler depuis les sources)

### Depuis les sources

```bash
git clone https://github.com/Nyx-Off/NyxIP.git
cd NyxIP
cargo build --release
```

L'executable se trouve dans `target/release/nyxip.exe` (~5 MB, portable).

### Depuis les releases

Telecharger le `.exe` depuis la page [Releases](https://github.com/Nyx-Off/NyxIP/releases).

---

## Utilisation

```bash
# Lancer l'interface graphique
nyxip.exe
```

1. La plage IP de votre reseau local est detectee automatiquement
2. Cliquez **Scan** ou appuyez sur Entree
3. Les resultats apparaissent en temps reel
4. Clic droit sur une IP pour les actions rapides (navigateur, RDP, SSH...)
5. **Exporter** genere un fichier `.xlsx` formate

### Formats de plage IP supportes

| Format | Exemple |
|---|---|
| CIDR | `192.168.1.0/24` |
| Plage dernier octet | `192.168.1.1-254` |
| Plage complete | `192.168.1.1-192.168.2.254` |
| IP unique | `192.168.1.1` |

### Menu contextuel (clic droit)

- Ouvrir dans le navigateur (HTTP, HTTPS, ports custom)
- Ouvrir dans l'explorateur de fichiers (partage SMB)
- Connexion Bureau a distance (RDP)
- Terminal SSH
- Copier l'IP / le MAC

---

## Architecture

```
src/
├── main.rs              # Point d'entree + icone
├── app.rs               # Logique applicative + export XLSX
├── ui/
│   ├── theme.rs         # Theme cyberpunk
│   ├── scan_panel.rs    # Controles de scan
│   └── results_table.rs # Tableau + menu contextuel
├── scanner/
│   ├── ping.rs          # ICMP (Windows API + fallback)
│   ├── arp.rs           # Resolution MAC (SendARP)
│   ├── dns.rs           # Reverse DNS
│   ├── ports.rs         # Scan TCP async
│   └── types.rs         # Structures de donnees
├── network/
│   ├── range.rs         # Parsing plages IP
│   └── interface.rs     # Detection interface locale
└── oui/
    └── database.rs      # Base OUI integree
```

### Stack technique

- **GUI** : [egui](https://github.com/emilk/egui) / eframe — rendu immediat, natif, leger
- **Async** : [Tokio](https://tokio.rs/) — runtime async pour scan concurrent
- **Reseau** : Windows API directe (`IcmpSendEcho`, `SendARP`) — pas de raw sockets, pas besoin d'admin
- **Export** : [rust_xlsxwriter](https://crates.io/crates/rust_xlsxwriter) — fichiers Excel natifs

---

## Licence

Ce projet est sous licence [MIT](LICENSE).

---

<p align="center">
  <strong>Cree par <a href="https://bensalem.dev">Samy Bensalem</a></strong><br/>
  <a href="https://github.com/Nyx-Off">@Nyx-Off</a>
</p>
