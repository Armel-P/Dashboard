export interface WidgetInstance {
  uid: string;
  service: string;
  widgetId: string;
  params: Record<string, string | number>;
  refreshSecs: number;
  locked: boolean;
  x?: number; y?: number;
  w: number; h: number; minW: number; minH: number;
}

export interface ServiceDefinition {
  name: string;
  label: string;
  auth: AuthKind;
  widgets: WidgetDefinition[];
}

export type AuthKind = 'none' | 'oauth2';

export interface WidgetDefinition {
  id: string;
  name: string;
  description: string;
  params: ParamDefinition[];
}

export interface ParamDefinition {
  name: string;
  type: 'string' | 'number' | 'integer';
  optional: boolean;
  enum?: string[];
  default?: unknown;
  minimum?: number;
  maximum?: number;
}
