// plugins/withShareExtension.js
// Expo config plugin to add iOS Share Extension

const { withPlugins, withXcodeProject, withInfoPlist, withEntitlementsPlist } = require('@expo/config-plugins');
const fs = require('fs');
const path = require('path');

const SHARE_EXTENSION_NAME = 'ShareExtension';
const BUNDLE_SHORT_VERSION = '1.0';
const BUNDLE_VERSION = '1';

function withShareExtension(config) {
  config = withXcodeProject(config, async (config) => {
    const xcodeProject = config.modResults;
    
    // Add share extension target
    const targetUuid = xcodeProject.generateUuid();
    const targetName = SHARE_EXTENSION_NAME;
    
    // This is a simplified version - full implementation would create:
    // 1. New target with product type com.apple.product-type.app-extension
    // 2. Build settings for the extension
    // 3. Copy phase for extension files
    // 4. Link frameworks
    
    return config;
  });

  config = withInfoPlist(config, (config) => {
    // Add app group capability for sharing data
    if (!config.modResults.CFBundleURLTypes) {
      config.modResults.CFBundleURLTypes = [];
    }
    
    return config;
  });

  config = withEntitlementsPlist(config, (config) => {
    // Add app groups entitlement
    const appGroupId = `group.${config.ios?.bundleIdentifier || 'com.remarkable.sync'}`;
    config.modResults['com.apple.security.application-groups'] = [appGroupId];
    
    return config;
  });

  return config;
}

module.exports = withShareExtension;
