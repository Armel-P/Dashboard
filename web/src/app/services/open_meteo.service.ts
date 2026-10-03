import { Injectable, inject } from "@angular/core";
import { Observable } from "rxjs";
import { WidgetService } from "./service.free";

export interface OpenMeteoPlace {
  name: string;
  country: string | null;
  latitude: number;
  longitude: number;
}

export interface CurrentWeather {
  place: OpenMeteoPlace;
  temperature: number;
  feels_like: number;
  wind_kmh: number;
  weather_code: number;
  units: 'celsius' | 'fahrenheit';
}

export interface ForecastDay {
  date: string;
  weather_code: number;
  temp_max: number;
  temp_min: number;
  precipitation_mm: number;
}

export interface DailyForecast {
  place: OpenMeteoPlace;
  units: 'celsius' | 'fahrenheit';
  days: ForecastDay[];
}

@Injectable({
  providedIn: 'root',
})
export class OpenMeteoService {
  private readonly widgets = inject(WidgetService);

  current(city: string, units = 'celsius') {
    return this.widgets.getWidget(
      'open_meteo',
      'current',
      { city, units },
    ) as Observable<CurrentWeather>;
  }

  forecast(city: string, units = 'celsius', days = 5) {
    return this.widgets.getWidget(
      'open_meteo',
      'forecast',
      { city, units, days },
    ) as Observable<DailyForecast>;
  }
}
