# wgpu-owo  

wgpu studies (game making)  

## 00_time

what it does:  

- draw a pixel art image of the sea.  
- also draw a cloud.  
- cloud moves with keyboard inputs.  
- pressing space pauses.  

what i did:  

- read the new version of learn-wgpu.  
- update with new api (wgpu 30.0).  
- update with better error handling.  
- changed to gl backend.  
- add delta time.  
- add world struct.  
- add pause by adding is_running field to World.  
- add need_redraw field to App to avoid unnecessary redraw.  

what i learnt:  

- my previous code was quite messy.  

reference:  

[Learn Wgpu](https://sotrh.github.io/learn-wgpu/)

## 01_space  

what it does:  

- similar to square from wgpu-uwu.  
- except the world coord is not NDC now.  

how it works:  

- world uses logical coord (x: 0 to LOGIC_WIDTH, y : 0 to LOGIC_HEIGHT).  
- uniform buffer has projection_matrix (convert logical coord to NDC).  
- uniform buffer has view_matrix (do nothing yet).  
- shader calculates movement and NDC with the matrices.  
- (projection \* model \* position)
- also, InputHandler is added to handle inputs.  

what i learnt:  

- pixel coord is much easier to work with.  

## 02_instances

what it does:  

- draw many rects with random position, size, and colour.  
- rects fall down with wavy move.  
- (rects are managed with rudimentary ecs system)

how it works:  

- world has components struct.  
- components has vec of position/velocity/size/colour.  
- each rects has id which also is the index of components vec.  
- world build a vec of InstanceData from components data.  
- renderer.update() write InstanceData to instance buffer.  
- shader use instance buffer data to draw many rects.  

what i learnt:  

- how to use instancing.  
- how to manage entities with ecs.  

## 03_grid

what it does:  

- draw colourful grids.  
- clicking changes the colour of grid.  

how it works:  

- grids are managed with ecs.  
- InputHandler converts click position to world coord.  
- ReactSystem converts world coord to grid coord and change the colour.  

what i learnt:  

- basics of grid system.  
- how to manage mouse inputs.  

## 04_sprites

what it does:  

- draw grids with flower sprites.  
- clicking changes the sprite.  

how it works:  

- world.rs has Sprite enum.  
- world has sprite_data array, which has SpriteData for each sprite.  
- SpriteData has uv_offset and uv_size.  
- world.build_instance_data use sprite_data array to add sprite data.  
- wgsl uses these data to sample the right area from sprite sheet.  

what i learnt:  

- basics of texture atlas.  
