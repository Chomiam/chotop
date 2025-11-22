# Guide de publication Chotop dans l'AUR

## ⚠️ Important: Les mises à jour NE SONT PAS automatiques

Contrairement à ce qu'on pourrait penser, **l'AUR ne se met PAS à jour automatiquement** quand tu publies une nouvelle release sur GitHub. Tu dois **mettre à jour manuellement** le PKGBUILD à chaque nouvelle version.

## Processus de publication (première fois)

### 1. Configuration SSH pour l'AUR

Si ce n'est pas déjà fait, configure ton accès SSH pour l'AUR:

```bash
# Génère une clé SSH si tu n'en as pas
ssh-keygen -t ed25519 -C "ton-email@example.com"

# Copie ta clé publique
cat ~/.ssh/id_ed25519.pub

# Va sur https://aur.archlinux.org/
# Connecte-toi avec ton compte
# Va dans "My Account" > "SSH Public Key"
# Colle ta clé publique
```

### 2. Clone le dépôt AUR (première fois seulement)

```bash
# Clone le dépôt vide (chotop-bin n'existe pas encore)
git clone ssh://aur@aur.archlinux.org/chotop-bin.git aur-chotop-bin
cd aur-chotop-bin

# Copie les fichiers
cp ../aur-package/PKGBUILD .
cp ../aur-package/.SRCINFO .

# Commit et push
git add PKGBUILD .SRCINFO
git commit -m "Initial upload: chotop-bin 1.0.2"
git push
```

### 3. Vérifie que c'est publié

Va sur https://aur.archlinux.org/packages/chotop-bin

## Processus de mise à jour (à chaque nouvelle release)

À **CHAQUE FOIS** que tu publies une nouvelle release GitHub (ex: v1.0.3):

### 1. Récupère le nouveau checksum

```bash
# Télécharge la nouvelle archive
wget https://github.com/Chomiam/chotop/releases/download/v1.0.3/chotop-v1.0.3-x86_64.tar.gz

# Calcule le checksum
sha256sum chotop-v1.0.3-x86_64.tar.gz
```

### 2. Met à jour le PKGBUILD

```bash
cd aur-chotop-bin

# Édite PKGBUILD
nano PKGBUILD
```

Change ces lignes:
```bash
pkgver=1.0.3          # ← Nouvelle version
pkgrel=1              # ← Remets à 1 pour nouvelle version
sha256sums=('...')    # ← Nouveau checksum
```

### 3. Régénère .SRCINFO

```bash
makepkg --printsrcinfo > .SRCINFO
```

### 4. Commit et push

```bash
git add PKGBUILD .SRCINFO
git commit -m "Update to 1.0.3"
git push
```

### 5. Les utilisateurs seront notifiés

Quand tu push la mise à jour:
- Les utilisateurs avec `yay` ou `paru` verront la nouvelle version disponible
- Ils pourront faire `yay -Syu` pour mettre à jour

## Workflow complet pour une nouvelle release

```bash
# 1. Dans overlay-daemon: Créer la release GitHub
git tag -a v1.0.3 -m "Release 1.0.3"
git push origin v1.0.3
cargo build --release
tar -czf chotop-v1.0.3-x86_64.tar.gz -C target/release discord-overlay-daemon chotop-config -C ../.. README.md install.sh
gh release create v1.0.3 --title "..." --notes "..." chotop-v1.0.3-x86_64.tar.gz

# 2. Calculer le checksum
sha256sum chotop-v1.0.3-x86_64.tar.gz

# 3. Mettre à jour l'AUR
cd ../aur-chotop-bin
nano PKGBUILD  # Mettre à jour pkgver et sha256sums
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 1.0.3"
git push
```

## Vérification locale avant de publier

Avant de push sur l'AUR, teste toujours le PKGBUILD localement:

```bash
cd aur-chotop-bin
makepkg -si  # Build et installe
```

Si ça marche, tu peux push en toute sécurité.

## Alternatives

### Option 1: chotop-bin (recommandé)
- Installation rapide depuis binaires pré-compilés
- Plus simple pour les utilisateurs
- C'est ce qu'on a créé ici

### Option 2: chotop (depuis sources)
- Compile depuis les sources GitHub
- Plus long mais plus "pur" pour certains utilisateurs
- Nécessite rust/cargo en dépendances de build

Tu peux publier les deux si tu veux !

## Liens utiles

- Guide officiel AUR: https://wiki.archlinux.org/title/AUR_submission_guidelines
- Compte AUR: https://aur.archlinux.org/
- Package chotop-bin (après publication): https://aur.archlinux.org/packages/chotop-bin
