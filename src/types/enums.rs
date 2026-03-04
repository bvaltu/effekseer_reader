//! All enum types used in Effekseer binary formats.
//!
//! Every enum is `#[non_exhaustive]`, derives `Debug, Clone, Copy, PartialEq, Eq`,
//! implements `From<i32>` (or `From<u8>`), and implements [`IsUnknown`].

use crate::reader::IsUnknown;

/// Helper macro to define an i32-based enum with `From<i32>`, `IsUnknown`, and standard derives.
macro_rules! define_enum_i32 {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $val:expr ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// Unknown discriminant value.
            Unknown(i32),
        }

        impl From<i32> for $name {
            fn from(v: i32) -> Self {
                match v {
                    $( $val => Self::$variant, )+
                    other => Self::Unknown(other),
                }
            }
        }

        impl IsUnknown for $name {
            fn is_unknown(&self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }
    };
}

/// Helper macro to define a u8-based enum with `From<u8>`, `IsUnknown`, and standard derives.
macro_rules! define_enum_u8 {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $val:expr ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// Unknown discriminant value.
            Unknown(u8),
        }

        impl From<u8> for $name {
            fn from(v: u8) -> Self {
                match v {
                    $( $val => Self::$variant, )+
                    other => Self::Unknown(other),
                }
            }
        }

        impl IsUnknown for $name {
            fn is_unknown(&self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }
    };
}

// ============================================================
// Core Enums
// ============================================================

define_enum_i32! {
    /// The type of an effect node in the node tree.
    EffectNodeType {
        /// Root node (always the top-level node).
        Root = -1,
        /// No-op node type.
        NoneType = 0,
        /// Sprite (billboard quad) renderer.
        Sprite = 2,
        /// Ribbon (connected strip) renderer.
        Ribbon = 3,
        /// Ring (torus/circle) renderer.
        Ring = 4,
        /// 3D model renderer.
        Model = 5,
        /// Track (trail) renderer.
        Track = 6,
    }
}

define_enum_i32! {
    /// Alpha blending mode for rendering.
    AlphaBlendType {
        /// Opaque (no blending).
        Opacity = 0,
        /// Standard alpha blend.
        Blend = 1,
        /// Additive blending.
        Add = 2,
        /// Subtractive blending.
        Sub = 3,
        /// Multiplicative blending.
        Mul = 4,
    }
}

define_enum_i32! {
    /// Billboard orientation mode.
    BillboardType {
        /// Face the camera.
        Billboard = 0,
        /// Lock Y axis, face camera horizontally.
        YAxisFixed = 1,
        /// No billboarding, use node transform.
        Fixed = 2,
        /// Billboard with rotation.
        RotatedBillboard = 3,
        /// Billboard aligned to velocity direction.
        DirectionalBillboard = 4,
    }
}

define_enum_i32! {
    /// Texture sampling filter mode.
    TextureFilterType {
        /// Nearest-neighbor filtering.
        Nearest = 0,
        /// Bilinear filtering.
        Linear = 1,
    }
}

define_enum_i32! {
    /// Texture wrap/addressing mode.
    TextureWrapType {
        /// Repeat tiling.
        Repeat = 0,
        /// Clamp to edge.
        Clamp = 1,
        /// Mirror tiling.
        Mirror = 2,
    }
}

define_enum_i32! {
    /// Parent transform binding mode.
    BindType {
        /// No binding to parent transform.
        NotBind = 0,
        /// Bind only at creation time.
        WhenCreating = 1,
        /// Bind continuously.
        Always = 2,
        /// No binding, but relative to root.
        NotBindRoot = 3,
    }
}

define_enum_i32! {
    /// Face culling mode.
    CullingType {
        /// Cull front faces.
        Front = 0,
        /// Cull back faces.
        Back = 1,
        /// No culling (double-sided).
        Double = 2,
    }
}

define_enum_i32! {
    /// Z-sort order for particle rendering.
    ZSortType {
        /// No z-sorting.
        None = 0,
        /// Normal (front-to-back) order.
        NormalOrder = 1,
        /// Reverse (back-to-front) order.
        ReverseOrder = 2,
    }
}

define_enum_i32! {
    /// Material type for renderers.
    RendererMaterialType {
        /// Default built-in material.
        Default = 0,
        /// Background distortion material.
        BackDistortion = 6,
        /// Lit (lighting-enabled) material.
        Lighting = 7,
        /// User-supplied material file.
        File = 128,
    }
}

define_enum_i32! {
    /// Shading model for materials.
    ShadingModelType {
        /// Lit (receives lighting).
        Lit = 0,
        /// Unlit (no lighting).
        Unlit = 1,
    }
}

define_enum_i32! {
    /// How texture color is interpreted.
    TextureColorType {
        /// RGBA color.
        Color = 0,
        /// Scalar value.
        Value = 1,
    }
}

define_enum_i32! {
    /// Shape used for culling.
    CullingShape {
        /// No culling shape.
        NoneShape = 0,
        /// Spherical culling volume.
        Sphere = 1,
    }
}

define_enum_i32! {
    /// How a model resource is referenced.
    ModelReferenceType {
        /// External model file.
        File = 0,
        /// Procedurally generated model.
        Procedural = 1,
        /// External reference.
        External = 2,
    }
}

define_enum_i32! {
    /// Trail smoothing mode.
    TrailSmoothingType {
        /// No smoothing.
        Off = 0,
        /// Smoothing enabled.
        On = 1,
    }
}

define_enum_i32! {
    /// Trail time evaluation mode.
    TrailTimeType {
        /// Use first particle's time.
        FirstParticle = 0,
        /// Use particle group's time.
        ParticleGroup = 1,
    }
}

define_enum_i32! {
    /// Ring shape type.
    RingShapeType {
        /// Donut shape.
        Donut = 0,
        /// Crescent shape.
        Crescent = 1,
    }
}

define_enum_i32! {
    /// Ring single-value parameter type.
    RingSingleType {
        /// Fixed value.
        Fixed = 0,
        /// Random range.
        Random = 1,
        /// Easing (ParameterEasingFloat).
        Easing = 2,
    }
}

define_enum_i32! {
    /// Ring location parameter type.
    RingLocationType {
        /// Fixed position.
        Fixed = 0,
        /// Position-velocity-acceleration.
        Pva = 1,
        /// Easing (old-style EasingVector2D).
        Easing = 2,
    }
}

define_enum_i32! {
    /// Track size parameter type.
    TrackSizeType {
        /// Fixed size.
        Fixed = 0,
    }
}

define_enum_i32! {
    /// Texture slot type for renderers.
    RendererTextureType {
        /// Color texture.
        Color = 0,
        /// Normal map texture.
        Normal = 1,
        /// Alpha texture.
        Alpha = 2,
        /// UV distortion texture.
        UVDistortion = 3,
        /// Blend texture.
        Blend = 4,
        /// Blend alpha texture.
        BlendAlpha = 5,
        /// Blend UV distortion texture.
        BlendUVDistortion = 6,
    }
}

define_enum_i32! {
    /// How UVs are mapped to texture coordinates.
    TextureUVType {
        /// Stretch to fit.
        Stretch = 0,
        /// Tile per particle.
        TilePerParticle = 1,
        /// Tile globally.
        Tile = 2,
    }
}

define_enum_i32! {
    /// Order in which instances are rendered.
    RenderingOrder {
        /// First created instance renders first.
        FirstCreatedInstanceIsFirst = 0,
        /// First created instance renders last.
        FirstCreatedInstanceIsLast = 1,
    }
}

define_enum_i32! {
    /// Whether fade-in is enabled.
    FadeInType {
        /// No fade-in.
        None = 0,
        /// Fade-in enabled.
        Use = 1,
    }
}

define_enum_i32! {
    /// Fade-out timing mode.
    FadeOutType {
        /// No fade-out.
        None = 0,
        /// Fade out within the particle's lifetime.
        WithinLifetime = 1,
        /// Fade out after the particle is removed.
        AfterRemoved = 2,
    }
}

// ============================================================
// CommonValues Enums
// ============================================================

define_enum_i32! {
    /// Parent bind type for translation, extending [`BindType`] with follow-parent variants.
    TranslationParentBindType {
        /// No binding.
        NotBind = 0,
        /// Bind at creation.
        WhenCreating = 1,
        /// Bind continuously.
        Always = 2,
        /// Not bound, relative to root.
        NotBindRoot = 3,
        /// Not bound, follow parent (triggers steering behavior).
        NotBindFollowParent = 4,
        /// Bind at creation, follow parent (triggers steering behavior).
        WhenCreatingFollowParent = 5,
    }
}

define_enum_i32! {
    /// When particle generation occurs.
    GenerationTiming {
        /// Generate continuously.
        Continuous = 0,
        /// Generate on trigger.
        Trigger = 1,
    }
}

define_enum_u8! {
    /// Trigger condition type.
    TriggerType {
        /// No trigger.
        None = 0,
        /// External trigger signal.
        ExternalTrigger = 1,
        /// Parent was removed.
        ParentRemoved = 2,
        /// Parent collided.
        ParentCollided = 3,
    }
}

// ============================================================
// Parameter Enums
// ============================================================

define_enum_i32! {
    /// Translation parameter type.
    ParameterTranslationType {
        /// Fixed position.
        Fixed = 0,
        /// Position-Velocity-Acceleration.
        Pva = 1,
        /// Easing interpolation.
        Easing = 2,
        /// F-Curve animation.
        FCurve = 3,
        /// NURBS curve path.
        NurbsCurve = 4,
        /// View-space offset.
        ViewOffset = 5,
        /// No translation.
        None = 0x7ffffffe,
    }
}

define_enum_i32! {
    /// NURBS curve loop behavior.
    NurbsCurveLoopType {
        /// Wrap around (fmod).
        Loop = 0,
        /// Clamp to end.
        Clamp = 1,
    }
}

define_enum_i32! {
    /// Rotation parameter type.
    ParameterRotationType {
        /// Fixed rotation.
        Fixed = 0,
        /// Position-Velocity-Acceleration.
        Pva = 1,
        /// Easing interpolation.
        Easing = 2,
        /// Axis-aligned PVA.
        AxisPva = 3,
        /// Axis-aligned easing.
        AxisEasing = 4,
        /// F-Curve animation.
        FCurve = 5,
        /// Rotate to face viewpoint.
        RotateToViewpoint = 6,
        /// Velocity-based rotation.
        Velocity = 7,
        /// No rotation.
        None = 0x7ffffffe,
    }
}

define_enum_i32! {
    /// Scaling parameter type.
    ParameterScalingType {
        /// Fixed scale.
        Fixed = 0,
        /// Position-Velocity-Acceleration.
        Pva = 1,
        /// Easing interpolation.
        Easing = 2,
        /// Single-axis PVA.
        SinglePva = 3,
        /// Single-axis easing.
        SingleEasing = 4,
        /// F-Curve animation.
        FCurve = 5,
        /// Single-axis F-Curve.
        SingleFCurve = 6,
        /// No scaling.
        None = 0x7ffffffe,
    }
}

define_enum_i32! {
    /// Spawn location generation type.
    GenerationLocationType {
        /// Spawn at a point.
        Point = 0,
        /// Spawn on a sphere surface.
        Sphere = 1,
        /// Spawn on a model surface.
        Model = 2,
        /// Spawn on a circle.
        Circle = 3,
        /// Spawn along a line.
        Line = 4,
    }
}

define_enum_i32! {
    /// How spawn positions are distributed on a model.
    ModelSpawnType {
        /// Random position.
        Random = 0,
        /// On vertices.
        Vertex = 1,
        /// Random vertex.
        VertexRandom = 2,
        /// On faces.
        Face = 3,
        /// Random face.
        FaceRandom = 4,
    }
}

define_enum_i32! {
    /// Coordinate space for model spawning.
    ModelCoordinateSpace {
        /// Parent coordinate space.
        Parent = 0,
        /// World coordinate space.
        World = 1,
    }
}

define_enum_i32! {
    /// Distribution pattern for circle spawning.
    CircleDistributionType {
        /// Random positions on circle.
        Random = 0,
        /// Ordered positions on circle.
        Order = 1,
        /// Reverse-ordered positions on circle.
        ReverseOrder = 2,
    }
}

define_enum_i32! {
    /// Axis for spawning direction.
    SpawnAxisType {
        /// X axis.
        X = 0,
        /// Y axis.
        Y = 1,
        /// Z axis.
        Z = 2,
    }
}

define_enum_i32! {
    /// Distribution pattern for line spawning.
    LineDistributionType {
        /// Random positions along line.
        Random = 0,
        /// Ordered positions along line.
        Order = 1,
    }
}

define_enum_i32! {
    /// Color parameter type for all-type color system.
    AllTypeColorType {
        /// Fixed color.
        Fixed = 0,
        /// Random color range.
        Random = 1,
        /// Easing between colors.
        Easing = 2,
        /// F-Curve RGBA animation.
        FCurveRgba = 3,
        /// Gradient over lifetime.
        Gradient = 4,
    }
}

define_enum_i32! {
    /// Custom data parameter type.
    ParameterCustomDataType {
        /// No custom data.
        None = 0,
        /// Fixed 2D value.
        Fixed2D = 20,
        /// Random 2D value.
        Random2D = 21,
        /// Easing 2D value.
        Easing2D = 22,
        /// F-Curve 2D value.
        FCurve2D = 23,
        /// Fixed 4D value.
        Fixed4D = 40,
        /// F-Curve color value.
        FCurveColor = 53,
        /// Dynamic input value.
        DynamicInput = 60,
    }
}

define_enum_i32! {
    /// UV animation type.
    UVAnimationType {
        /// Default UV mapping.
        Default = 0,
        /// Fixed UV coordinates.
        Fixed = 1,
        /// Animated UV frames.
        Animation = 2,
        /// Scrolling UV.
        Scroll = 3,
        /// F-Curve UV animation.
        FCurve = 4,
    }
}

define_enum_i32! {
    /// Loop behavior for UV animation.
    UVAnimationLoopType {
        /// Play once.
        Once = 0,
        /// Loop.
        Loop = 1,
        /// Loop with reverse.
        ReverseLoop = 2,
    }
}

define_enum_i32! {
    /// Interpolation mode for UV animation frames.
    UVAnimationInterpolationType {
        /// No interpolation (snap to frame).
        None = 0,
        /// Linear interpolation between frames.
        Lerp = 1,
    }
}

define_enum_i32! {
    /// Alpha cutoff parameter type.
    AlphaCutoffType {
        /// Fixed threshold.
        Fixed = 0,
        /// Four-point interpolation.
        FourPointInterpolation = 1,
        /// Easing.
        Easing = 2,
        /// F-Curve.
        FCurve = 3,
    }
}

define_enum_i32! {
    /// Kill rule type for particles.
    KillType {
        /// No kill rule.
        None = 0,
        /// Kill outside a box.
        Box = 1,
        /// Kill on one side of a plane.
        Plane = 2,
        /// Kill outside a sphere.
        Sphere = 3,
    }
}

define_enum_i32! {
    /// Coordinate system for collision detection.
    WorldCoordinateSystemType {
        /// Local (parent) coordinate system.
        Local = 0,
        /// Global (world) coordinate system.
        Global = 1,
    }
}

define_enum_i32! {
    /// Sound parameter type.
    ParameterSoundType {
        /// No sound.
        None = 0,
        /// Sound enabled.
        Use = 1,
    }
}

define_enum_i32! {
    /// Sound panning mode.
    ParameterSoundPanType {
        /// 2D panning.
        Pan2D = 0,
        /// 3D positional panning.
        Pan3D = 1,
    }
}

define_enum_i32! {
    /// Behavior when a particle doesn't match the active LOD level.
    NonMatchingLODBehaviour {
        /// Hide the particle.
        Hide = 0,
        /// Don't spawn new particles.
        DontSpawn = 1,
        /// Don't spawn and hide existing.
        DontSpawnAndHide = 2,
    }
}

define_enum_i32! {
    /// Easing function type (newer system, v16+).
    Easing3Type {
        /// Cubic speed curve using stored (a, b, c) parameters.
        StartEndSpeed = 0,
        /// Linear interpolation.
        Linear = 1,
        /// Quadratic ease-in.
        EaseInQuadratic = 10,
        /// Quadratic ease-out.
        EaseOutQuadratic = 11,
        /// Quadratic ease-in-out.
        EaseInOutQuadratic = 12,
        /// Cubic ease-in.
        EaseInCubic = 20,
        /// Cubic ease-out.
        EaseOutCubic = 21,
        /// Cubic ease-in-out.
        EaseInOutCubic = 22,
        /// Quartic ease-in.
        EaseInQuartic = 30,
        /// Quartic ease-out.
        EaseOutQuartic = 31,
        /// Quartic ease-in-out.
        EaseInOutQuartic = 32,
        /// Quintic ease-in.
        EaseInQuintic = 40,
        /// Quintic ease-out.
        EaseOutQuintic = 41,
        /// Quintic ease-in-out.
        EaseInOutQuintic = 42,
        /// Back ease-in (overshoots).
        EaseInBack = 50,
        /// Back ease-out (overshoots).
        EaseOutBack = 51,
        /// Back ease-in-out (overshoots).
        EaseInOutBack = 52,
        /// Bounce ease-in.
        EaseInBounce = 60,
        /// Bounce ease-out.
        EaseOutBounce = 61,
        /// Bounce ease-in-out.
        EaseInOutBounce = 62,
    }
}

define_enum_u8! {
    /// Color mode for random color ranges.
    ColorMode {
        /// RGBA color space.
        Rgba = 0,
        /// HSVA color space.
        Hsva = 1,
    }
}

// ============================================================
// Force Field Enums
// ============================================================

define_enum_i32! {
    /// Local force field type (non-sequential values).
    LocalForceFieldType {
        /// No force field.
        None = 0,
        /// Turbulence (noise-based).
        Turbulence = 1,
        /// Directional force.
        Force = 2,
        /// Wind force.
        Wind = 3,
        /// Vortex force.
        Vortex = 4,
        /// Drag force.
        Drag = 7,
        /// Gravity force.
        Gravity = 8,
        /// Attractive force toward a point.
        AttractiveForce = 9,
    }
}

define_enum_i32! {
    /// Vortex force field sub-type.
    ForceFieldVortexType {
        /// Constant angular velocity.
        ConstantAngle = 0,
        /// Constant linear speed.
        ConstantSpeed = 1,
    }
}

define_enum_i32! {
    /// Turbulence force field sub-type.
    ForceFieldTurbulenceType {
        /// Simple noise.
        Simple = 0,
        /// Complicated noise.
        Complicated = 1,
    }
}

define_enum_i32! {
    /// Force field falloff shape type.
    LocalForceFieldFalloffType {
        /// No falloff.
        None = 0,
        /// Spherical falloff.
        Sphere = 1,
        /// Tube falloff.
        Tube = 2,
        /// Cone falloff.
        Cone = 3,
    }
}

// ============================================================
// Other Enums
// ============================================================

define_enum_i32! {
    /// Falloff blend mode.
    FalloffBlendType {
        /// Additive falloff.
        Add = 0,
        /// Subtractive falloff.
        Sub = 1,
        /// Multiplicative falloff.
        Mul = 2,
    }
}

define_enum_i32! {
    /// Procedural model generation type.
    ProceduralModelType {
        /// Mesh-based procedural model.
        Mesh = 0,
        /// Ribbon-based procedural model.
        Ribbon = 1,
    }
}

define_enum_i32! {
    /// Primitive shape for mesh procedural models.
    ProceduralModelPrimitiveType {
        /// Sphere shape.
        Sphere = 0,
        /// Cone shape.
        Cone = 1,
        /// Cylinder shape.
        Cylinder = 2,
        /// Spline4 shape.
        Spline4 = 3,
    }
}

define_enum_i32! {
    /// Cross-section type for ribbon procedural models.
    ProceduralModelCrossSectionType {
        /// Planar cross-section.
        Plane = 0,
        /// Cross-shaped cross-section.
        Cross = 1,
        /// Point cross-section.
        Point = 2,
    }
}

define_enum_i32! {
    /// Axis type for procedural models.
    ProceduralModelAxisType {
        /// X axis.
        X = 0,
        /// Y axis.
        Y = 1,
        /// Z axis.
        Z = 2,
    }
}

define_enum_i32! {
    /// F-Curve edge behavior at start/end.
    FCurveEdge {
        /// Constant (hold the value).
        Constant = 0,
        /// Loop the curve.
        Loop = 1,
        /// Loop with offset.
        LoopInversely = 2,
    }
}

// ============================================================
// GPU Particle Enums (u8)
// ============================================================

define_enum_u8! {
    /// GPU particle emit shape.
    GpuEmitShape {
        /// Point emission.
        Point = 0,
        /// Line emission.
        Line = 1,
        /// Circle emission.
        Circle = 2,
        /// Sphere emission.
        Sphere = 3,
        /// Model-surface emission.
        Model = 4,
    }
}

define_enum_u8! {
    /// GPU particle scale type.
    ///
    /// Note: `Pva` (1) is defined in the C++ enum but not implemented in the loader.
    /// The C++ runtime asserts false for this value.
    GpuScaleType {
        /// Fixed scale.
        Fixed = 0,
        /// PVA scale (unimplemented in C++ — will error).
        Pva = 1,
        /// Easing scale.
        Easing = 2,
    }
}

define_enum_u8! {
    /// GPU particle color parameter type.
    GpuColorParamType {
        /// Fixed color.
        Fixed = 0,
        /// Random color range.
        Random = 1,
        /// Easing between colors.
        Easing = 2,
        /// F-Curve color animation.
        FCurve = 3,
        /// Gradient color.
        Gradient = 4,
    }
}

define_enum_u8! {
    /// GPU particle color space.
    GpuColorSpaceType {
        /// RGBA color space.
        Rgba = 0,
        /// HSVA color space.
        Hsva = 1,
    }
}

define_enum_u8! {
    /// GPU particle render shape.
    GpuRenderShape {
        /// Billboard sprite.
        Sprite = 0,
        /// 3D model.
        Model = 1,
        /// Trail.
        Trail = 2,
    }
}

define_enum_u8! {
    /// GPU particle material type.
    GpuMaterialType {
        /// Unlit material.
        Unlit = 0,
        /// Lit material.
        Lighting = 1,
    }
}

// ============================================================
// Material Enums
// ============================================================

define_enum_i32! {
    /// Required predefined method type in materials.
    RequiredPredefinedMethodType {
        /// Gradient method.
        Gradient = 0,
        /// Noise method.
        Noise = 1,
        /// Light method.
        Light = 2,
        /// Local time method.
        LocalTime = 3,
        /// HSV method.
        Hsv = 4,
        /// Particle time method.
        ParticleTime = 5,
    }
}

define_enum_i32! {
    /// Material uniform value type.
    MaterialValueType {
        /// Single float.
        Float1 = 0,
        /// Two floats.
        Float2 = 1,
        /// Three floats.
        Float3 = 2,
        /// Four floats.
        Float4 = 3,
    }
}

// ============================================================
// RemovalTiming bitflags
// ============================================================

bitflags::bitflags! {
    /// Bitflags indicating when a particle should be removed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct RemovalTiming: i32 {
        /// No removal.
        const NONE = 0;
        /// Remove when life reaches zero.
        const WHEN_LIFE_IS_EXTINCT = 1;
        /// Remove when parent is removed.
        const WHEN_PARENT_IS_REMOVED = 2;
        /// Remove when all children are extinct.
        const WHEN_CHILDREN_IS_EXTINCT = 4;
        /// Remove when triggered.
        const WHEN_TRIGGERED = 8;
    }
}
