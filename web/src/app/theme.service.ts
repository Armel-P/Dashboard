import { Injectable, inject } from '@angular/core';
import { DOCUMENT } from '@angular/common';

export enum Theme {
  Light = 'light',
  Dark = 'dark'
}

const themes = {
  [Theme.Light]: {
    favicon: 'dashboard.ico'
    // logo: 'assets/images/logo-light.svg'
  },

  [Theme.Dark]: {
    favicon: 'dashboard.ico'
    // logo: 'assets/images/logo-dark.svg'
  }
};

@Injectable({
  providedIn: 'root'
})
export class ThemeService {
  private readonly document = inject(DOCUMENT);
  private readonly storageKey = 'theme';

  constructor() {
    const savedTheme = localStorage.getItem(this.storageKey) as Theme | null;

    if (savedTheme === Theme.Light || savedTheme === Theme.Dark) {
      this.setTheme(savedTheme);
    } else {
      this.setTheme(Theme.Dark);
    }
  }

  get currentTheme(): Theme {
    return (
      this.document.documentElement.dataset['theme'] as Theme
    ) || Theme.Dark;
  }

  setTheme(theme: Theme): void {
    this.document.documentElement.dataset['theme'] = theme;

    localStorage.setItem(this.storageKey, theme);

    this.updateFavicon(theme);
  }

  toggle(): void {
    this.setTheme(
      this.currentTheme === Theme.Dark
        ? Theme.Light
        : Theme.Dark
    );
  }

  private updateFavicon(theme: Theme): void {
    let link = this.document.querySelector<HTMLLinkElement>(
      'link[rel="icon"]'
    );

    if (!link) {
      link = this.document.createElement('link');

      link.rel = 'icon';

      this.document.head.appendChild(link);
    }

    link.href = themes[theme].favicon;
  }
}
