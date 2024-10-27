self:
{ lib
, pkgs
, config
, ...
}:
let
  lure = self.packages.${pkgs.system}.default;

  cfg = config.services.lure;

  supportedServices = [ "lastfm" "listenbrainz" ];

  commonServiceOptions = with lib;
    service: {
      username = mkOption {
        type = types.str;
        description = "${if service == "lastfm" then "Last.fm" else "ListenBrainz"} username to check for listening activity.";
      };

      check_interval = mkOption {
        type = types.int;
        description = "Interval in seconds to check for listening activity.";
        default = 16;
      };
    };
in
{
  options.services.lure = with lib; {
    enable = mkEnableOption "Enable lure service.";

    package = mkOption {
      type = types.package;
      description = "The lure package to use.";
      default = lure;
    };

    log = mkOption {
      type = types.nullOr (types.str);
      description = "`[EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)` compatible filter for log messages.";
      default = null;
    };

    useService = mkOption {
      type = types.nullOr (types.enum supportedServices);
      description = "Which service to enable for checking your listening status.";
      default = null;
    };

    services = {
      lastfm = mkOption {
        type = types.nullOr (types.submodule {
          options = commonServiceOptions "lastfm" // {
            api_key = {
              type = with types; either str path;
              description = ''
                The API key to use for the Last.fm API.

                WARNING: Since this key basically gives full access to the API
                over YOUR account, it is recommended to use a path to a file that
                contains the key, instead of entering the key as a string, so it's
                stored securely.
              '';
            };
          };
        });
        default = null;
        description = "Options for the Last.fm service.";
      };

      listenbrainz = mkOption {
        type = types.nullOr (types.submodule {
          options = commonServiceOptions "listenbrainz" // {
            api_url = mkOption {
              type = types.str;
              description = "The API URL of the ListenBrainz instance.";
              default = "https://api.listenbrainz.org";
            };
          };
        });
        default = null;
        description = "Options for the ListenBrainz service.";
      };
    };

    revolt = {
      status = {
        template = mkOption {
          type = types.str;
          description = ''
            The status template that will be used.

            The following placeholders can be used:
            - %NAME%: The name of the song.
            - %ARTIST%: The artist of the song.
          '';
          default = "🎵 Listening to %NAME% by %ARTIST%";
        };

        idle = mkOption {
          type = types.nullOr types.str;
          description = ''
            The idle status.
            
            If this option is not set, the status will be returned to
            the previous status when not listening to anything.
          '';
          default = null;
        };
      };

      api_url = mkOption {
        type = types.str;
        description = "The API URL of the instance.";
        default = "https://api.revolt.chat";
      };

      session_token = mkOption {
        type = with types; either str path;
        description = ''
          The `X-Session-Token` to use for the API.

          WARNING: Since this token basically gives full access to your account,
          it is recommended to use a path to a file that contains the token,
          instead of entering the token as a string, so it's stored securely.
        '';
      };
    };
  };

  config = with lib; mkIf (cfg.enable && cfg.useService != null) {
    assertions = [
      {
        assertion = cfg.useService == "lastfm" -> cfg.services.lastfm != null;
        message = "`services.lastfm` options must be provided when using LastFM service.";
      }
      {
        assertion = cfg.useService == "listenbrainz" -> cfg.services.listenbrainz != null;
        message = "`services.listenbrainz` options must be provided when using ListenBrainz service.";
      }
    ];

    warnings = [
      (mkIf (isString cfg.revolt.session_token) "'revolt.session_token' is specified as a string, PLEASE consider using a path to a file instead for the sake of security.")
    ];

    systemd.services.lure = {
      description = "Lure service";

      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      startLimitIntervalSec = 10;
      startLimitBurst = 5;

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/lure start";
        Restart = "on-failure";
        RestartSec = "5s";
        LoadCredential =
          let
            credentials = [ ]
              ++ lib.optional (lib.isPath cfg.lastfm.api_key) "lastfm-api-key:${cfg.lastfm.api_key}"
              ++ lib.optional (lib.isPath cfg.revolt.session_token) "revolt-session-token:${cfg.revolt.session_token}";
          in
          credentials;
      };

      environment = with lib; mkMerge [
        {
          LURE_ENABLE = cfg.useService;

          LURE_REVOLT__STATUS__TEMPLATE = builtins.replaceStrings [ "%" ] [ "%%" ] cfg.revolt.status.template;
          LURE_REVOLT__API_URL = cfg.revolt.api_url;
        }
        (optionalAttrs (cfg.log != null) {
          LURE_LOG = cfg.log;
        })
        (optionalAttrs (cfg.useService == "lastfm") mkMerge [
          {
            LURE_LASTFM__USERNAME = cfg.lastfm.username;
            LURE_LASTFM__CHECK_INTERVAL = toString cfg.lastfm.check_interval;
          }
          (optionalAttrs (isString cfg.lastfm.api_key) {
            LURE_LASTFM__API_KEY = cfg.lastfm.api_key;
          })
          (optionalAttrs (isPath cfg.lastfm.api_key) {
            LURE_LASTFM__API_KEY_FILE = "%d/lastfm-api-key";
          })
        ])
        (optionalAttrs (cfg.useService == "listenbrainz") {
          LURE_LISTENBRAINZ__USERNAME = cfg.listenbrainz.username;
          LURE_LISTENBRAINZ__CHECK_INTERVAL = toString cfg.listenbrainz.check_interval;
          LURE_LISTENBRAINZ__API_URL = cfg.listenbrainz.api_url;
        })
        (optionalAttrs (cfg.revolt.status.idle != null) {
          LURE_REVOLT__STATUS__IDLE = builtins.replaceStrings [ "%" ] [ "%%" ] cfg.revolt.status.idle;
        })
        (optionalAttrs (isString cfg.revolt.session_token) {
          LURE_REVOLT__SESSION_TOKEN = cfg.revolt.session_token;
        })
        (optionalAttrs (isPath cfg.revolt.session_token) {
          LURE_REVOLT__SESSION_TOKEN_FILE = "%d/revolt-session-token";
        })
      ];
    };
  };
}
