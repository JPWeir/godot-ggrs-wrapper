extends Sprite2D

func change_color():
	self.modulate = Color.RED

func _on_body_entered(body: Node) -> void:
	change_color()
