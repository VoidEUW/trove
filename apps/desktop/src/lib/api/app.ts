import type { AppInfo } from '../bindings/AppInfo';
import { call } from './invoke';

/** The app's version and this installation's device id. */
export function getAppInfo(): Promise<AppInfo> {
	return call<AppInfo>('app_info');
}
