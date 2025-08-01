import { Window as KeplrWindow } from '@keplr-wallet/types';
declare global {
    interface Window extends KeplrWindow {
    }
}
export default function deployWithKeplr(): Promise<void>;
//# sourceMappingURL=deploy.d.ts.map