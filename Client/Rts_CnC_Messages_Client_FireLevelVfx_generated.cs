using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_FireLevelVfx
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.FireLevelVfx); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.FireLevelVfx)obj;
            //  Serialize EventId
            s.Write(value.EventId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Orientation
            s.Write(value.Orientation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.FireLevelVfx)) as Rts.CnC.Messages.Client.FireLevelVfx;
            //  Deserialize EventId
            s.Read(out value.EventId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Orientation
            s.Read(out value.Orientation);

            return value;
        }
        
    }
}
