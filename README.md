# godot-ggrs-wrapper

The purpose of this repo is to create a wrapper for the [ggrs](https://github.com/gschup/ggrs) Rust crate so that this can be used inside the Godot Game Engine. To accomplish this the [godot-rust](https://github.com/godot-rust/godot-rust) GDExtension bindings are used. Although the project currently implements all of the GGRS features, **I can't guarantee** that it's ready for use in production.

### Now forked for Godot 4
I forked and updated this repo to work with Godot 4. With interface changes to both Godot and GGRS, it seems like some additional work will be needed to provide this wrapper in an easily configurable package.

For now, I'm committing this on a dev branch as a proof of concept (mostly maintaining the existing interface, with some small changes), but I plan to redesign this wrapper's interface in the near future.

## Notes

- It was once recommended that you set `reloadable` to `false` inside the `GDExtension` Godot resource (i.e. `godot-ggrs-wrapper.gdextension`).
  - This may or may not still be recommended.
- Tested on Godot version 4.4.1 with a local P2PSession
- Inputs in the example are currently in the format of a unsigned 8-bit integer, as all permutations of 4 directional inputs can be represented in 4 bits.
- States are PackedByteArrays, you convert the godot variant to a ByteArray and give it to GodotGGRS.

## Links
TODO: Update these links to this fork's docs, once/if published
- [Documentation](https://docs.rs/godot-ggrs-wrapper/)
- [Crates.io](https://crates.io/crates/godot-ggrs-wrapper)

## Examples

TODO: Update the demo video with an example from Godot 4
- [Remote connection between 2 different machines](https://youtu.be/8gwXVMqhNs8)
- [Example Project](https://github.com/jpweir/godot-ggrs-wrapper/tree/main/example)

## Projects that use godot-ggrs-wrapper

Does your project use godot-ggrs-wrapper? Let me know and i'll be happy to add a link!

- [Girls With Swords](https://github.com/trian-gles/Girls-with-swords-GGRS)

## Quick start

Since this project uses [godot-rust](https://github.com/godot-rust/godot-rust) GDExtension bindings you should be familiar with how to set up a godot-rust GDExtension node inside Godot. If you aren't please check out [this page](https://godot-rust.github.io/book/intro/index.html) for details. The setup you want to have is a scene with a node that has the GodotGgrsP2PSession class bound to it.

### Starting a session with GodotGgrsP2PSession

To set up a session you want to have a script that addresses the GGRS node. Just having the node inside your scene doesn't create a session for you. Here I took the root node to make a script that makes a session on the **\_ready()** function

```gdscript
func _ready():
    var local_port: int # Identifies the local port
	var local_handle: int #This var is later used to identify your local inputs
	var remote_handle: int #This var is later used to identify the remote_players inputs
	if(OS.get_cmdline_args()[0] == "listen"):
		local_handle = $GodotGGRS.add_local_player()
		remote_handle = $GodotGGRS.add_remote_player("127.0.0.1:7071")
		local_port = 7070
	elif(OS.get_cmdline_args()[0] == "join"):
		local_port = 7071
		remote_handle = $GodotGGRS.add_remote_player("127.0.0.1:7070")
		local_handle = $GodotGGRS.add_local_player()
	$GodotGGRS.set_input_delay(2) # Set personal frame_delay, sets only for local handles.
	$GodotGGRS.set_max_prediction_window(8)
	$GodotGGRS.set_callback_node(self) # Set the node which will implement the callback methods
	$GodotGGRS.start_session(local_port)
```

As you can see we swap the order of adding players depending on who's the "host". In reality since it's a peer 2 peer library, there is no true host. However you should have a way to distinguish between player 1 and player 2.

### Advancing frames

Now that we have a session we want to start implementing our loop. Godot's default **\_process()** and **\_physics_process()** will serve us nicely here.

```gdscript
func _process(_delta):
	$GodotGGRS.poll_remote_clients() # GGRS needs to periodically process UDP requests and such, sticking it in \_process() works nicely since it's only called on idle.

func _physics_process(_delta):
	if($GodotGGRS.is_running()): # This will return true when all players and spectators have joined and have been synched.
		$GodotGGRS.advance_frame(local_handle, raw_input_to_int("con1")) # raw_input_to_int is a method that parses InputActions that start with "con1" into a integer.

func raw_input_to_int(prefix: String)->int:
	# This method is how I parse InputActions into an int, but as long as it's an int it doesn't matter how it's parsed.
	var result := 0;
	if(Input.is_action_pressed(prefix + "_left")): #The action it checks here would be "con1_left" if the prefix is set to "con1"
		result |= 1
	if(Input.is_action_pressed(prefix + "_right")):
		result |= 2
	if(Input.is_action_pressed(prefix + "_up")):
		result |= 4
	if(Input.is_action_pressed(prefix + "_down")):
		result |= 8
	return result;
```

Calling advance_frame will tell GGRS that you are ready to go to the next frame using the input you've given as a parameter. GGRS will do it's thing and callback to Godot once it's ready to continue.

### Handling GGRS callbacks

So how to handle GGRS callbacks is alot more subjective than the steps before and will vary greatly on how your game is built. The only thing required is that you implement the callback functions, but the logic inside can be pretty much anything to fit to your game. Here's how i implemented the callback methods.

```gdscript
func ggrs_advance_frame(inputs: Array):
	# inputs is an array of input data indexed by handle.
	# input_data itself is an integer

	var net1_inputs := 0;
	var net2_inputs := 0;
	if(local_handle < remote_handle):
		net1_inputs = inputs[local_handle]
		net2_inputs = inputs[remote_handle]
	else:
		net1_inputs = inputs[remote_handle]
		net2_inputs = inputs[local_handle]
	int_to_raw_input("net1", net1_inputs) # Player objects check for InputActions that aren't bound to any controller.
	int_to_raw_input("net2", net2_inputs) # Player objects check for InputActions that aren't bound to any controller.
	_handle_player_frames()

func ggrs_load_game_state(frame: int, buffer: PoolByteArray, checksum: int):
	var state : Dictionary = bytes2var(buffer);
	P1.load_state(state.get("P1", {}))
	P2.load_state(state.get("P2", {}))

func ggrs_save_game_state(frame: int)->PoolByteArray: # frame parameter can be used as a sanity check (making sure it matches your internal frame counter).
	var save_state = {}
	save_state["P1"] = _save_P1_state("");
	save_state["P2"] = _save_P2_state("");
	return var2bytes(save_state);

func int_to_raw_input(prefix: String, inputs: int):
	_set_action(prefix + "_left", inputs & 1)
	_set_action(prefix + "_right", inputs & 2)
	_set_action(prefix + "_up", inputs & 4)
	_set_action(prefix + "_down", inputs & 8)

func _set_action(action: String, pressed: bool):
	if(pressed):
		Input.action_press(action)
	else:
		Input.action_release(action)

```
