import { Type } from "@angular/core";
import { OpenMeteoCurrent} from "./open_meteo_current/open_meteo_current";
// import { OpenMeteoForecast } from "./open_meteo_forecast/open_meteo_forecast";

export interface WidgetEntry {
  service: string;
  id: string;
  component: Type<unknown>;
  w: number; h: number;
  minW: number; minH: number;
  defaultRefreshSecs: number;
}

export const WIDGET_REGISTRY: WidgetEntry[] = [
  { service: 'open_meteo', id: 'current',  component: OpenMeteoCurrent,
    w: 4, h: 3, minW: 2, minH: 3, defaultRefreshSecs: 600 },
//   { service: 'open_meteo', id: 'forecast', component: OpenMeteoForecast, w: 6, h: 4,
// minW: 1, minH: 1, defaultRefreshSecs: 3600 },
]; //  future implementation, back done, just front needed

export const SERVICE_NAMES: string[] = [...new Set(WIDGET_REGISTRY.map(e => e.service))];
