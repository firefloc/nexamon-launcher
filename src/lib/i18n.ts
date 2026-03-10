import { writable, derived } from "svelte/store";

export type Locale = "fr" | "en";

function loadLocale(): Locale {
  try {
    const saved = localStorage.getItem("nexamon_locale");
    if (saved === "en" || saved === "fr") return saved;
  } catch {}
  return "fr";
}

export const locale = writable<Locale>(loadLocale());

// Persist on change
locale.subscribe((v) => {
  try { localStorage.setItem("nexamon_locale", v); } catch {}
});

const translations: Record<Locale, Record<string, string>> = {
  fr: {
    // Sidebar
    "nav.play": "Jouer",
    "nav.settings": "Parametres",
    "nav.console": "Console",
    "nav.logout": "Deconnexion",

    // Login
    "login.subtitle": "Lanceur Minecraft",
    "login.button": "Se connecter avec Microsoft",
    "login.browser_opened": "Une fenetre de navigateur s'est ouverte. Entrez ce code :",
    "login.copy_hint": "Cliquez pour copier",
    "login.copied": "Copie !",
    "login.open_again": "Rouvrir la page de connexion",
    "login.waiting": "En attente d'authentification...",

    // Main
    "main.select_profile": "Profil",
    "main.no_profiles": "Aucun profil configure. Ajoutez-en un dans les Parametres.",
    "main.not_installed": "Non installe",
    "main.play": "Jouer",
    "main.running": "En cours",
    "main.checking_java": "Verification Java...",
    "main.downloading_java": "Telechargement Java...",
    "main.downloading_minecraft": "Telechargement Minecraft...",
    "main.installing_fabric": "Installation Fabric...",
    "main.syncing_mods": "Synchronisation mods...",
    "main.launching": "Lancement...",
    "main.hero_wip": "Fonctionnalite en cours de developpement — bientot vous pourrez suivre votre progression ici !",
    "main.repair_title": "Verifier et reparer",
    "main.open_folder": "Ouvrir le dossier",

    // Config dialog
    "dialog.config_update": "Mise a jour de configuration",
    "dialog.required_updates": "Mises a jour obligatoires appliquees",
    "dialog.force_updated": "Certaines configurations ont ete mises a jour pour la compatibilite.",
    "dialog.optional_single": "1 fichier de configuration que vous avez modifie a aussi ete mis a jour.",
    "dialog.optional_plural": "{n} fichiers de configuration que vous avez modifies ont aussi ete mis a jour.",
    "dialog.keep_settings": "Garder mes reglages",
    "dialog.use_defaults": "Utiliser les parametres du pack",
    "dialog.more": "+{n} de plus...",

    // Uninstall dialog
    "dialog.uninstall": "Desinstaller le pack",
    "dialog.uninstall_confirm": "Cela supprimera tous les fichiers telecharges pour ce profil. Vous pourrez le reinstaller plus tard.",
    "dialog.cancel": "Annuler",
    "dialog.uninstall_btn": "Desinstaller",
    "dialog.ok": "OK",
    "dialog.error": "Erreur",
    "dialog.install_failed": "L'installation a echoue",

    // Repair dialog
    "dialog.repair_failed": "Reparation echouee",
    "dialog.pack_repaired": "Pack repare",
    "dialog.pack_clean": "Pack en ordre",
    "dialog.no_issues": "Aucun probleme detecte. Tous les mods, datapacks et configs correspondent au pack attendu.",
    "dialog.removed_mods": "Supprime {n} mod(s) non autorise(s)",
    "dialog.removed_datapacks": "Supprime {n} datapack(s) non autorise(s)",
    "dialog.restored_configs": "Restaure {n} config(s)",

    // Profile card
    "profile.installed": "Installe",
    "profile.install": "Installer",
    "profile.uninstall": "Desinstaller",
    "profile.ram_recommended": "RAM recommandee",
    "profile.ram_insufficient": "RAM insuffisante pour ce pack",

    // RAM warning dialog
    "dialog.ram_warning": "RAM insuffisante",
    "dialog.ram_warning_text": "Votre systeme ne dispose pas de suffisamment de RAM pour ce profil ({recommended} GB de RAM recommandes, {available} GB disponibles). Le lanceur allouera le maximum de memoire RAM disponible au jeu.",
    "dialog.ram_continue": "Continuer",
    "dialog.ram_dismiss": "Ne plus afficher cette alerte",

    // Maintenance
    "nav.maintenance": "Maintenance",
    "maintenance.configs": "Configuration",
    "maintenance.tools": "Outils",

    // Settings
    "settings.title": "Parametres",
    "settings.performance": "Performance",
    "settings.ram": "Allocation RAM",
    "settings.java": "Java",
    "settings.java_path": "Chemin Java (laisser vide pour auto-detection)",
    "settings.auto_detect": "Auto-detection",
    "settings.launcher": "Lanceur",
    "settings.language": "Langue",
    "settings.close_on_launch": "Fermer le lanceur au demarrage du jeu",
    "settings.auto_accept_configs": "Accepter automatiquement les mises a jour de configuration",
    "settings.save": "Enregistrer",
    "settings.saved": "Enregistre",
    "settings.maintenance": "Maintenance",
    "settings.repair": "Verifier et reparer",
    "settings.repair_desc": "Supprime les mods non autorises et restaure les fichiers manquants ou modifies.",
    "settings.repairing": "Reparation...",
    "settings.open_folder": "Ouvrir le dossier",
    "settings.open_folder_desc": "Ouvre le dossier d'instance Minecraft dans l'explorateur de fichiers.",

    // Console
    "console.title": "Console",
    "console.auto_scroll": "Defilement auto",
    "console.clear": "Effacer",
    "console.empty": "Aucune sortie. Lancez le jeu pour voir les logs.",

    // Update
    "update.available": "Mise a jour v{version} disponible",
    "update.now": "Mettre a jour",
    "update.downloading": "Telechargement...",
    "update.restarting": "Redemarrage...",

    // Dev
    "nav.dev": "Dev",
    "dev.title": "Developpeur",
    "dev.layout_debug": "Debug layout",
    "dev.show_outlines": "Afficher les outlines de debug",
  },

  en: {
    "nav.play": "Play",
    "nav.settings": "Settings",
    "nav.console": "Console",
    "nav.logout": "Logout",

    "login.subtitle": "Minecraft Launcher",
    "login.button": "Sign in with Microsoft",
    "login.browser_opened": "A browser window has opened. Enter this code:",
    "login.copy_hint": "Click to copy",
    "login.copied": "Copied!",
    "login.open_again": "Open login page again",
    "login.waiting": "Waiting for authentication...",

    "main.select_profile": "Profile",
    "main.no_profiles": "No profiles configured. Add one in Settings.",
    "main.not_installed": "Not installed",
    "main.play": "Play",
    "main.running": "Running",
    "main.checking_java": "Checking Java...",
    "main.downloading_java": "Downloading Java...",
    "main.downloading_minecraft": "Downloading Minecraft...",
    "main.installing_fabric": "Installing Fabric...",
    "main.syncing_mods": "Syncing mods...",
    "main.launching": "Launching...",
    "main.hero_wip": "Feature under development — soon you'll be able to track your progress here!",
    "main.repair_title": "Verify & repair",
    "main.open_folder": "Open folder",

    "dialog.config_update": "Configuration update",
    "dialog.required_updates": "Required updates applied",
    "dialog.force_updated": "Some configs were force-updated for compatibility.",
    "dialog.optional_single": "1 config file you customized has also been updated.",
    "dialog.optional_plural": "{n} config files you customized have also been updated.",
    "dialog.keep_settings": "Keep my settings",
    "dialog.use_defaults": "Use pack defaults",
    "dialog.more": "+{n} more...",

    "dialog.uninstall": "Uninstall pack",
    "dialog.uninstall_confirm": "This will delete all downloaded files for this profile. You can reinstall it later.",
    "dialog.cancel": "Cancel",
    "dialog.uninstall_btn": "Uninstall",
    "dialog.ok": "OK",
    "dialog.error": "Error",
    "dialog.install_failed": "Installation failed",

    "dialog.repair_failed": "Repair failed",
    "dialog.pack_repaired": "Pack repaired",
    "dialog.pack_clean": "Pack is clean",
    "dialog.no_issues": "No issues found. All mods, datapacks, and configs match the expected pack.",
    "dialog.removed_mods": "Removed {n} unauthorized mod(s)",
    "dialog.removed_datapacks": "Removed {n} unauthorized datapack(s)",
    "dialog.restored_configs": "Restored {n} config(s)",

    "profile.installed": "Installed",
    "profile.install": "Install",
    "profile.uninstall": "Uninstall",
    "profile.ram_recommended": "Recommended RAM",
    "profile.ram_insufficient": "Insufficient RAM for this pack",

    "dialog.ram_warning": "Insufficient RAM",
    "dialog.ram_warning_text": "Your system does not have enough RAM for this profile ({recommended} GB RAM recommended, {available} GB available). The launcher will allocate the maximum available RAM.",
    "dialog.ram_continue": "Continue",
    "dialog.ram_dismiss": "Don't show this warning again",

    "nav.maintenance": "Maintenance",
    "maintenance.configs": "Configuration",
    "maintenance.tools": "Tools",

    "settings.title": "Settings",
    "settings.performance": "Performance",
    "settings.ram": "RAM Allocation",
    "settings.java": "Java",
    "settings.java_path": "Java Path (leave empty for auto-detect)",
    "settings.auto_detect": "Auto-detect",
    "settings.launcher": "Launcher",
    "settings.language": "Language",
    "settings.close_on_launch": "Close launcher when game starts",
    "settings.auto_accept_configs": "Automatically accept config updates",
    "settings.save": "Save Settings",
    "settings.saved": "Saved",
    "settings.maintenance": "Maintenance",
    "settings.repair": "Verify & repair",
    "settings.repair_desc": "Removes unauthorized mods and restores missing or modified files.",
    "settings.repairing": "Repairing...",
    "settings.open_folder": "Open folder",
    "settings.open_folder_desc": "Opens the Minecraft instance folder in your file manager.",

    "console.title": "Console",
    "console.auto_scroll": "Auto-scroll",
    "console.clear": "Clear",
    "console.empty": "No log output yet. Launch the game to see logs.",

    "update.available": "Update v{version} available",
    "update.now": "Update now",
    "update.downloading": "Downloading...",
    "update.restarting": "Restarting...",

    "nav.dev": "Dev",
    "dev.title": "Developer",
    "dev.layout_debug": "Layout debug",
    "dev.show_outlines": "Show debug outlines",
  },
};

/** Reactive translation function */
export const t = derived(locale, ($locale) => {
  return (key: string, params?: Record<string, string | number>): string => {
    let text = translations[$locale][key] ?? translations.en[key] ?? key;
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  };
});
