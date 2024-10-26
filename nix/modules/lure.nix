{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.lure;

  readSecretOrValue = value:
    if (isPath value || (isString value && hasPrefix "/" value)) then
      "$(cat ${value})"
    else
      value;

  # Common options that both services share
  commonServiceOptions = {
    username = mkOption {
      type = types.str;
      description = "Username to check for listening activity.";
    };
    check_interval = mkOption {
      type = types.int;
      default = 16;
      description = "Interval in seconds to check for listening activity.";
    };
  };

  # Create conditional options based on active service
  serviceOptions = {
    lastfm = types.submodule {
      options = commonServiceOptions // {
        api_key = mkOption {
          type = types.nullOr (types.either types.str types.path);
          default = null;
          description = "Last.fm API key or path to file containing the key.";
        };
        api_key_file = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "Path to file containing the Last.fm API key.";
        };
      };
    };

    listenbrainz = types.submodule {
      options = commonServiceOptions // {
        api_url = mkOption {
          type = types.str;
          default = "https://api.listenbrainz.org";
          description = "ListenBrainz API URL.";
        };
      };
    };
  };

in {
  options.services.lure = {
    enable = mkEnableOption "lure music status service";

    activeService = mkOption {
      type = types.enum [ "lastfm" "listenbrainz" ];
      description =
        "Which service to enable for checking your listening status.";
      example = "lastfm";
    };

    services = mkOption {
      type = types.submodule {
        options = {
          lastfm = mkOption {
            type = serviceOptions.lastfm;
            description = "Last.fm service configuration.";
            default = null;
          };
          listenbrainz = mkOption {
            type = serviceOptions.listenbrainz;
            description = "ListenBrainz service configuration.";
            default = null;
          };
        };
      };
      description = "Service-specific configurations.";
    };

    revolt = {
      status = {
        template = mkOption {
          type = types.str;
          default = "🎵 Listening to %NAME% by %ARTIST%";
          description =
            "Status template with placeholders for song information.";
        };

        idle = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Idle status message when not listening to music.";
        };
      };

      api_url = mkOption {
        type = types.str;
        default = "https://api.revolt.chat";
        description = "Revolt API URL.";
      };

      session_token = mkOption {
        type = types.nullOr (types.either types.str types.path);
        default = null;
        description = "Revolt session token or direct value.";
      };

      session_token_file = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to file containing the Revolt session token.";
      };
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.activeService == "lastfm" -> cfg.services.lastfm
          != null;
        message = "Last.fm service is active but configuration is missing.";
      }
      {
        assertion = cfg.activeService == "listenbrainz"
          -> cfg.services.listenbrainz != null;
        message =
          "ListenBrainz service is active but configuration is missing.";
      }
      {
        assertion = cfg.services.lastfm != null -> (cfg.services.lastfm.api_key
          != null || cfg.services.lastfm.api_key_file != null);
        message =
          "Last.fm service configuration requires either api_key or api_key_file to be set.";
      }
      {
        assertion = cfg.services.lastfm != null -> !(cfg.services.lastfm.api_key
          != null && cfg.services.lastfm.api_key_file != null);
        message =
          "Cannot specify both api_key and api_key_file for Last.fm configuration.";
      }
      {
        assertion = !(cfg.revolt.session_token != null
          && cfg.revolt.session_token_file != null);
        message =
          "Cannot specify both session_token and session_token_file for Revolt configuration.";
      }
    ];

    systemd.services.lure = {
      description = "Lure music status service";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = "/bin/lure start";
        Restart = "always";
        RestartSec = "10";
        #DynamicUser = true;
        StandardError = "journal";
        StandardOutput = "journal";
        StandardInput = "null";
        LoadCredential = let
          credentials = [ ] ++ optional (cfg.services.lastfm != null
            && cfg.services.lastfm.api_key_file != null)
            "lastfm-api-key:${cfg.services.lastfm.api_key_file}"
            ++ optional (cfg.revolt.session_token_file != null)
            "revolt-session-token:${cfg.revolt.session_token_file}";
        in credentials;
      };

      environment = let
        # Last.fm environment variables
        lastfmEnv = optionalAttrs (cfg.services.lastfm != null) {
          LURE_SERVICES__LASTFM__USERNAME = cfg.services.lastfm.username;
          LURE_SERVICES__LASTFM__CHECK_INTERVAL =
            toString cfg.services.lastfm.check_interval;
          LURE_SERVICES__LASTFM__API_KEY =
            optionalString (cfg.services.lastfm.api_key != null)
            (readSecretOrValue cfg.services.lastfm.api_key);
          LURE_SERVICES__LASTFM__API_KEY_FILE =
            optionalString (cfg.services.lastfm.api_key_file != null)
            "%d/lastfm-api-key";
        };

        # ListenBrainz environment variables
        listenbrainzEnv = optionalAttrs (cfg.services.listenbrainz != null) {
          LURE_SERVICES__LISTENBRAINZ__USERNAME =
            cfg.services.listenbrainz.username;
          LURE_SERVICES__LISTENBRAINZ__API_URL =
            cfg.services.listenbrainz.api_url;
          LURE_SERVICES__LISTENBRAINZ__CHECK_INTERVAL =
            toString cfg.services.listenbrainz.check_interval;
        };
      in {
        LURE_ENABLE = cfg.activeService;
        LURE_LOG = "trace";

        # Revolt configuration
        LURE_REVOLT__STATUS__TEMPLATE = cfg.revolt.status.template;
        LURE_REVOLT__STATUS__IDLE = cfg.revolt.status.idle;
        LURE_REVOLT__API_URL = cfg.revolt.api_url;
        LURE_REVOLT__SESSION_TOKEN =
          optionalString (cfg.revolt.session_token != null)
          (readSecretOrValue cfg.revolt.session_token);
        LURE_REVOLT__SESSION_TOKEN_FILE =
          optionalString (cfg.revolt.session_token_file != null)
          "%d/revolt-session-token";
      } // lastfmEnv // listenbrainzEnv;
    };
  };
}
