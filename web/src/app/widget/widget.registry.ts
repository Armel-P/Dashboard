import { Type } from "@angular/core";
import { OpenMeteoCurrent} from "./open_meteo_current/open_meteo_current";
import { OpenMeteoForecast } from "./open_meteo_forecast/open_meteo_forecast";
import { SoundCloudPlaylist } from './soundcloud_playlist/soundcloud_playlist';

export interface WidgetEntry {
  service: string;
  id: string;
  component: Type<unknown>;
  w: number; h: number;
  minW: number; minH: number;
  defaultRefreshSecs: number;
}

export const FREE_WIDGET_REGISTRY: WidgetEntry[] = [
  { service: 'open_meteo', id: 'current',  component: OpenMeteoCurrent,
    w: 4, h: 3, minW: 2, minH: 3, defaultRefreshSecs: 600 },
  { service: 'open_meteo', id: 'forecast', component: OpenMeteoForecast,
    w: 6, h: 3, minW: 2, minH: 4, defaultRefreshSecs: 3600 },
];

export const FREE_SERVICE_NAMES: string[] = [...new Set(FREE_WIDGET_REGISTRY.map(e => e.service))];

export const OAUTH_WIDGET_REGISTRY: WidgetEntry[] = [
  { service: 'soundcloud', id: 'playlist', component: SoundCloudPlaylist,
    w: 6, h: 5, minW: 3, minH: 4, defaultRefreshSecs: 3600,
  },
];

export const OAUTH_SERVICE_NAMES: string[] = [...new Set(OAUTH_WIDGET_REGISTRY.map(e => e.service))];

export const WIDGET_REGISTRY: WidgetEntry[] = FREE_WIDGET_REGISTRY.concat(OAUTH_WIDGET_REGISTRY);

export const SERVICE_NAMES: string[] = [...new Set(WIDGET_REGISTRY.map(e => e.service))];
