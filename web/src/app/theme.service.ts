import { DOCUMENT } from '@angular/common'; import { Injectable, inject } from '@angular/core';

export enum Theme {
  Light = 'light',
  Dark = 'dark',
  Kermit = 'kermit',
  Go = 'go'
}

const themes: Record<Theme, { favicon: string }> = {
  [Theme.Light]: { favicon: 'dashboard.ico' },
  [Theme.Dark]: { favicon: 'dashboard.ico' },
  [Theme.Kermit]: { favicon: 'kermit.ico' },
  [Theme.Go]: { favicon: 'dashboard.ico' }
};

const themeOrder: Theme[] = [Theme.Dark, Theme.Light, Theme.Kermit, Theme.Go];

@Injectable({ providedIn: 'root' })
export class ThemeService {
  private readonly document = inject(DOCUMENT);
  private readonly storageKey = 'theme';

  constructor() {
    this.setTheme(this.getSavedTheme());
  }

  get currentTheme(): Theme {
    const theme = this.document.documentElement.dataset['theme'];

    return this.isTheme(theme) ? theme : Theme.Dark;
  }

  setTheme(theme: Theme): void {
    if (!this.isTheme(theme))
      theme = Theme.Dark;

    this.document.documentElement.dataset['theme'] = theme;

    try {
      localStorage.setItem(this.storageKey, theme);
    } catch {}

    this.updateFavicon(theme);
  }

  toggle(): void {
    const currentIndex = themeOrder.indexOf(this.currentTheme);
    const nextIndex = (currentIndex + 1) % themeOrder.length;

    this.setTheme(themeOrder[nextIndex]);
  }

  private getSavedTheme(): Theme {
    try {
      const savedTheme = localStorage.getItem(this.storageKey);

      if (this.isTheme(savedTheme))
        return savedTheme;
    } catch {}

    return Theme.Dark;
  }

  private isTheme(value: string | null | undefined): value is Theme {
    return Object.values(Theme).includes(value as Theme);
  }

  private updateFavicon(theme: Theme): void {
    let link = this.document.querySelector<HTMLLinkElement>( 'link[rel="icon"]' );
    if (!link)
      link = this.document.createElement('link'); link.rel = 'icon'; this.document.head.appendChild(link);

    link.href = themes[theme].favicon;
  }
}
