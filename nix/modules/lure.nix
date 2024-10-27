{ lib
, pkgs
, config
  # Inputs
, lurePackage
, ...
}:
let
  cfg = config.services.lure;

  readSecretOrValue = value:
    if (lib.isPath value) then
      lib.readFile value
    else
      value;

  supportedServices = [ "lastfm" "listenbrainz" ];

  commonServiceOptions = with lib;
    service: {
      _module.check = mkMerge [
        (assertMsg (elem service supportedServices)
          "Service must be either 'lastfm' or 'listenbrainz', got '${service}'")
      ];

      username = mkOption {
        type = types.str;
        description = "${if service == "lastfm" then "Last.fm" else "ListenBrainz"} username to check for listening activity.";
      };

      check_interval = mkOption {
        type = types.int;
        default = 16;
        description = "Interval in seconds to check for listening activity.";
      };
    };
in
{
  options.services.lure = with lib; {
    enable = mkEnableOption "Enable lure service.";

    package = mkOption {
      type = types.package;
      description = "The lure package to use.";
      default = lurePackage;
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

    # services = {
    # };

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
    warnings = [
      (mkIf (isString cfg.revolt.session_token) "Revolt session token is specified as a string, PLEASE consider using a path to a file instead.")
    ];

    systemd.services.lure = {
      description = "Lure service"; # TODO: Fix this

      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      startLimitIntervalSec = 10;
      startLimitBurst = 5;

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/lure start";
        Restart = "on-failure";
        RestartSec = "5s";
        # LoadCredential = 
      };

      environment = { } // lib.optionalAttrs (!lib.isNull cfg.log) {
        RUST_LOG = cfg.log;
      };
    };
  };
}
