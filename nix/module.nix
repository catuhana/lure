{ config, pkgs, lib }: {
  options.services.lure =
    let
      cfg = config.services.lure;
    in
    with lib;
    {
      enable = mkEnableOption "lure";

      package = mkPackageOption pkgs "lure" { };

      platforms = mkOption {
        type = attrValues {
          lastfm = attrValues {
            enable = mkEnableOption "last.fm platform.";

            options = attrValues {
              user = mkOption {
                type = nullOr types.str;
                default = null;
                description = "Last.fm username to check current listening status from.";
              };

              api_key = mkOption {
                type = nullOr types.str;
                default = null;
                description = "Last.fm API key to be able to check current listening through API.";
              };

              check_interval = mkOption {
                type = types.int;
                default = 12;
                description = "Interval in seconds to check current listening status.";
              };
            };
          };

          listenbrainz = attrValues {
            enable = mkEnableOption "ListenBrainz platform.";

            options = attrValues {
              user = mkOption {
                type = nullOr types.str;
                default = null;
                description = "Listenbrainz username to check current listening status from.";
              };

              api_url = mkOption {
                type = types.str;
                default = "https://api.listenbrainz.org";
                description = "ListenBrainz API URL to send the API requests to";
              };

              check_interval = mkOption {
                type = types.int;
                default = 12;
                description = "Interval in seconds to check current listening status.";
              };
            };
          };
        };
      };

      sessionToken = mkOption {
        type = nullOr types.str;
        default = null;
        description = mdDoc "Session token for lure. DO NOT USE THIS OPTION, USE `sessionTokenFile` INSTEAD.";
      };

      sessionTokenFile = mkOption {
        type = nullOr types.path;
        default = null;
        description = "Session token file for lure.";
      };

      status = mkOption {
        type = attrValues {
          template = mkOption { type = types.str; default = "🎵 Listening to %NAME% by %ARTIST%"; };
          idle = mkOption { type = nullOr types.str; default = null; };
        };
      };

      configDir = mkOption {
        type = types.path;
        default = "/var/lib/lure/config.toml";
        description = "Config folder for lure.";
      };
    };
}
