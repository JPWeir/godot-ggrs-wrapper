extends RigidBody2D

@export var speed: int = 5
@export var force: int = 0

func _ready():
	$Name.text = name

# Velocity and constant forces are left out of the state intentionally.
# See what happens in a sync test session if you try to add a force without saving it.
func save_state() -> Vector2:
	return position
	
func load_state(state: Vector2):
	position = state

func up():
	if force > 0:
		add_constant_central_force(Vector2(0, -1 * force))
	position.y -= speed
	
func down():
	if force > 0:
		add_constant_central_force(Vector2(0, force))
	position.y += speed
	
func right():
	if force > 0:
		add_constant_central_force(Vector2(force, 0))
	position.x += speed

func left():
	if force > 0:
		add_constant_central_force(Vector2(-1 * force, 0))
	position.x -= speed
	
	 
