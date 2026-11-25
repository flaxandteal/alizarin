// What importing @alizarin/geo-napi registers with the host (see register.js).
// Kept apart from register.js so a test can run it against the host it has
// loaded, rather than a copy of it.
const binding = require('./index.js')

function register(host) {
  host.registerExtensionHandler(binding.geoHandlerPtr())
  host.registerDeriveProvider(binding.utmDeriveProviderPtr())
}

module.exports = { binding, register }
