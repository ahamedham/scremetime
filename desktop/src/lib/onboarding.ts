const ONBOARDED_KEY = "scremetime.onboarded";

export function hasOnboarded(): boolean {
  try {
    return localStorage.getItem(ONBOARDED_KEY) === "true";
  } catch {
    // Storage can throw in some contexts (private browsing style
    // restrictions); treat that the same as "not onboarded yet" rather
    // than crashing the app over a non-essential preference.
    return false;
  }
}

export function markOnboarded(): void {
  try {
    localStorage.setItem(ONBOARDED_KEY, "true");
  } catch {
    // Nothing meaningful to do if storage is unavailable; onboarding
    // will simply show again next launch.
  }
}
