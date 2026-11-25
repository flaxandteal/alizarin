// What importing @alizarin/clm-napi registers with the host (see register.js).
// Kept apart from register.js so a test can run it against the host it has
// loaded, rather than a copy of it.
const binding = require('./index.js')

function register(host) {
  host.registerExtensionHandler(binding.referenceHandlerPtr())
  host.registerExtensionMutation(binding.referenceChangeCollectionPtr())
}

module.exports = { binding, register }
