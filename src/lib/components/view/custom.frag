uniform vec4 surfaceColor;

varying vec3 pos;

void main()
{
    // Compute face normal using fragment position derivatives
    vec3 dx = dFdx(pos);
    vec3 dy = dFdy(pos);
    vec3 normal = normalize(cross(dx, dy));

    // Make light move with the camera for solid shading effect
    vec3 viewDir = normalize(cameraPosition - pos);
    vec3 lightDir = normalize(viewDir); 

    // Compute diffuse lighting - use absolute value to handle both front and back faces
    float diffuse = max(abs(dot(normal, lightDir)), 0.1); // Add ambient minimum

    // Soft rim light effect
    float rim = pow(1.0 - abs(dot(viewDir, normal)), 3.0);

    // Merge colors
    vec3 baseColor = surfaceColor.xyz;
    vec3 shadedColor = baseColor * diffuse + vec3(rim * 0.2);
    
    gl_FragColor = vec4(shadedColor, 1.0);
}
