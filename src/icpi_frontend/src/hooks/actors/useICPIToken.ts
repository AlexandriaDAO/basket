import { createActorHook } from 'ic-use-actor';
import { _SERVICE } from '../../../declarations/ICPI/ICPI.did';
import { canisterId, idlFactory } from '../../../declarations/ICPI';

const useICPIToken = createActorHook<_SERVICE>({
  canisterId: canisterId!,
  idlFactory: idlFactory,
});

export default useICPIToken;
