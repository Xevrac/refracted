using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestLocationBuild
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestLocationBuild); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestLocationBuild)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize FactoryEntityId
            s.Write(value.FactoryEntityId);
            //  Serialize EntityBeneathPlayerId
            s.Write(value.EntityBeneathPlayerId);
            //  Serialize EntityBeneathId
            s.Write(value.EntityBeneathId);
            //  Serialize EntityType
            s.Write(value.EntityType);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Orientation
            s.Write(value.Orientation);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestLocationBuild)) as Rts.CnC.Messages.Client.RequestLocationBuild;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize FactoryEntityId
            s.Read(out value.FactoryEntityId);
            //  Deserialize EntityBeneathPlayerId
            s.Read(out value.EntityBeneathPlayerId);
            //  Deserialize EntityBeneathId
            s.Read(out value.EntityBeneathId);
            //  Deserialize EntityType
            s.Read(out value.EntityType);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Orientation
            s.Read(out value.Orientation);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
