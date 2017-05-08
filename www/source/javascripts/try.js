//= require vendor/ansi_up
//= require vendor/underscore.min
//= require vendor/codemirror/codemirror
//= require vendor/codemirror/matchbrackets
//= require vendor/codemirror/shell
//= require vendor/josh/killring
//= require vendor/josh/history
//= require vendor/josh/readline
//= require vendor/josh/shell
//= require vendor/josh/pathhandler

/*global $, ansi_up, CodeMirror, Josh */

// Without this, you cannot open the keyboard on mobile devices since josh.js
// does not use actual HTML input elements.
$("#mobile-keyboard-trigger").click(function() {
  $(this).focus();
});

(function () {
    function format(text) {
        return "<pre>" + ansi_up.ansi_to_html(ansi_up.escape_for_html(text)) +
            "</pre>";
    }

    function getResponse(name) {
        return $.get("/try/responses/" + name + ".txt");
    }

    function success(display) {
        $("#success .button").removeClass("secondary").addClass("cta");
        if(display === "hide") {
          $("#shell-cli").hide();
        }
    }

    var editor;
    var editorEl = document.getElementById("try-habitat-editor");
    var history = new Josh.History({ key: 'helloworld.history'});
    var inStudio = false;
    var promptCounter = 1;
    var rootPrompt = "user@workstation-machine:~$";
    var shell = Josh.Shell({ history: history });
    var step = parseInt($("#try-habitat-progress").data("step"), 10);
    var studioPrompt = "<span class='ansi-green'>[</span>" +
        "<span class='ansi-cyan'>" + promptCounter +
        "</span><span class='ansi-green'>][habitat:</span>" +
        "<span class='ansi-magenta'>/src</span>" +
        "<span class='ansi-green'>:</span>0<span class='ansi-green'>]$</span>";
    var initialPrompt = step === 0 ? rootPrompt : studioPrompt;

    shell.setCommandHandler("exit", {
        exec: function(cmd, args, callback) {
            inStudio = false;
            shell.setPrompt(rootPrompt);
            shell.onNewPrompt(function(callback) {
                callback(rootPrompt);
            });
            $("#shell-cli").parent().empty();
            callback();
        }
    });

    // studio commands
    shell.setCommandHandler("studio", {
        exec: function(cmd, args, callback) {

            // studio enter
            if (args[0] === "enter") {
                getResponse("studio-enter").then(function (txt) {
                    inStudio = true;
                    callback(format(txt));
                    shell.setPrompt(studioPrompt);
                    shell.onNewPrompt(function(callback) {
                        promptCounter += 1;
                        callback(studioPrompt);
                    });
                    if (step === 1) { success(); }
                });

            // studio help
            } else if (args[0] === "help") {
                getResponse("studio-help").then(function (txt) {
                    callback(format(txt));
                });

            // studio <unsupported>
            } else {
                getResponse("studio-help").then(function (txt) {
                    callback(format(txt) + "<br>In this shell, only the " +
                        "<em>enter</em> subcommand is available. See " +
                        "<a target='_blank' href='#'>the documentation</a> " +
                        "to see what you can use in an actual shell.<br><br>");
                });
            }
        },
        completion: function(cmd, arg, line, callback) {
            callback(shell.bestMatch(arg, ["enter", "build", "help", "new",
                "rm", "run", "version"]));
        }
    });

    // hab commands
    shell.setCommandHandler("hab", {
        exec: function(cmd, args, callback) {

          switch(args[0]) {
             case "start":
                   // Start a service
                  if (args[1] === "example/ruby-rails-sample") {
                    getResponse("hab-start-service").then(function (txt) {
                        inStudio = false;
                        callback(format(txt));
                        shell.setPrompt(rootPrompt);
                        shell.onNewPrompt(function(callback) {
                            promptCounter += 1;
                            callback(rootPrompt);
                        });
                        
                        if (step === 1) { success(); }
                    });
                  } else if (args[1] + ' ' + args[2] === "--peer 172.17.0.2 --bind database:postgresql.default") {
                    getResponse("hab-bind").then(function (txt) {
                        inStudio = false;
                        callback(format(txt));
                        shell.setPrompt(rootPrompt);
                        shell.onNewPrompt(function(callback) {
                            promptCounter += 1;
                            callback(rootPrompt);
                        });
                        // step is pulled from the progress bar attribute in the DOM
                        if (step === 2) { success(); }
                     });
                  } else
                    getResponse("hab-start-help").then(function (txt) {
                        callback(format(txt));
                  });
                  break;
             case "config":
                // Apply service group configuration
                // inject the config.toml into the group
                if (args.join(" ") === "config apply --peer 172.17.0.2 ruby-rails-sample.default 1 fix_db_conn.toml") {
                  getResponse("hab-config-apply").then(function (txt) {
                      inStudio = false;
                      callback(format(txt));
                      shell.setPrompt(rootPrompt);
                      shell.onNewPrompt(function(callback) {
                          promptCounter += 1;
                          callback(rootPrompt);
                      });
                      // step is pulled from the progress bar attribute in the DOM
                      if (step === 3) {
                        success();

                        //change button text to reflect resulting status
                        //show badge on window buttons
                        //show full updated cli output
                        $(".node-status").html("change applied").parent().addClass("updated");
                        $(".button-badge, .full-output").show();
                      }
                  });
                } else 
                   getResponse("hab-config-apply-help").then(function (txt) {
                       callback(format(txt));
                   });
                break;
             default:
               // the user entered `hab` followed by an unsupported subcommand let's show them `hab help` for a list of available emulator subcommands and link to docs for the full subcommand list
               getResponse("hab-help").then(function (txt) {
                   callback(format(txt) + "Note: In this demo shell, only a " +
                       "few subcommands are available.<br>See the " +
                       "<a target='_blank' href='/docs/overview/'>Habitat documentation</a> " +
                       "for a more complete liste of features.<br><br>");
               });
          }
        //     // hab start [subcommand]
        //     if (args[0] === "blarg") {

        //         } else if (args[1] + ' ' + args[2] === "--peer 172.17.0.2 --bind database:postgresql.default") {
        //           getResponse("hab-bind").then(function (txt) {
        //               inStudio = false;
        //               callback(format(txt));
        //               shell.setPrompt(rootPrompt);
        //               shell.onNewPrompt(function(callback) {
        //                   promptCounter += 1;
        //                   callback(rootPrompt);
        //               });
        //               // step is pulled from the progress bar attribute in the DOM
        //               if (step === 2) { success(); }
        //           });

        //         // Adding a leader/follower topology
        //         } else if (args[1] + ' ' + args[2] === "-t leader") {
        //           if (args[3] === 'core/redis') {

        //             // adding the first/leader node
        //             if ((args[4] == null) && (step === 6)) {
        //               getResponse("hab-start-first-node").then(function (txt) {
        //                   inStudio = true;
        //                   callback(format(txt));
        //                   shell.setPrompt(rootPrompt);
        //                   shell.onNewPrompt(function(callback) {
        //                       promptCounter += 1;
        //                       callback(rootPrompt);
        //                   });

        //                   if (step === 6) { success('hide'); }
        //               });

        //             // adding an additional/follower node
        //             } else if (args[4] + ' ' + args[5] === "--peer 172.17.0.4") {
        //               getResponse("hab-start-additional-node").then(function (txt) {
        //                   inStudio = true;
        //                   callback(format(txt));
        //                   shell.setPrompt(rootPrompt);
        //                   shell.onNewPrompt(function(callback) {
        //                       promptCounter += 1;
        //                       callback(rootPrompt);
        //                   });
        //                   //change button text to reflect resulting status
        //                   //show badge on window buttons
        //                   //show full updated cli output
        //                   if (step === 7) {
        //                     success();

        //                     $(".node-status").html("connected").parent().addClass("updated");
        //                     $(".button-badge, .full-output").show();
        //                   }
        //               });

        //             // they could be on step 6 or 7 since the command/subcommand is the same
        //             } else if (step === 7) {
        //               getResponse("hab-follower-help").then(function (txt) {
        //                   callback(format(txt));
        //               });

        //             } else {
        //               getResponse("hab-leader-help").then(function (txt) {
        //                   callback(format(txt));
        //               });
        //             };
        //           } else {
        //             getResponse("hab-leader-help").then(function (txt) {
        //                 callback(format(txt));
        //             });
        //           }
        //         // if user tries to start anything else, then show the 'hab start' help
        //         } else {
        //           getResponse("hab-start-help").then(function (txt) {
        //               callback(format(txt));
        //           });
        //         };
        // }
       },
    });
    shell.setPrompt(rootPrompt);
    shell.activate();

    // Text Editor steps via CodeMirror
    if (editorEl) {
        editor = CodeMirror.fromTextArea(editorEl, {
            mode: "shell",
            lineNumbers: true,
            matchBrackets: true,
        });

        editor.on("changes", function (instance, changes) {
            if (step === 4 &&
                instance.getValue().match(/tcp-backlog\s\=\s*128\s*/)) {
                success();
            }
        });
    }

    // Switching windows when adding services
    $(".window-buttons .button").click(function(event) {
        var targetID = $(this).attr("data-target");

        // set button classes
        $(".window-buttons .button").removeClass("active");
        $(this).addClass("active");

        // show/hide windows
        $(".window-node").hide();
        $("#" + targetID).show();
    });

    // Hack to allow pasting.
    // See https://github.com/sdether/josh.js/issues/28
    $("#shell-panel").pastableNonInputable();
    $("#shell-panel").on("pasteText", function (event, data) {
        shell.addText(data.text);
    });
}());
