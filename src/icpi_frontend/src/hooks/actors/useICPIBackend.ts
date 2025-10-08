import { createActorHook } from 'ic-use-actor';
import { _SERVICE } from 'declarations/icpi_backend/icpi_backend.did';
import { canisterId, idlFactory } from 'declarations/icpi_backend';

const useICPIBackend = createActorHook<_SERVICE>({
  canisterId: canisterId!,
  idlFactory: idlFactory,
});

export default useICPIBackend;
