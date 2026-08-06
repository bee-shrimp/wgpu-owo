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
- add delta time.  
- add world struct.  
- add pause by adding is_running field to World.  
- add need_redraw field to App to avoid unnessesary redraw.  

what i learnt:  

- my previous code was quite messy.  


reference:  

[Learn Wgpu](https://sotrh.github.io/learn-wgpu/)

## 01_space  

what it does:  

- same as 00_time.  
- except the world coord is not NDC now.  

how it works:  

- world uses logical coord (x: 0 to LOGIC_WIDTH,y : 0 to LOGIC_HEIGHT).  
- uniform buffer has matrices needed for converting coord to NDC.  
- shader calculates movement and NDC with the matrices.  
