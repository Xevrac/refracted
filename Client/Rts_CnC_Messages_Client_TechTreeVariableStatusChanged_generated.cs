using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeVariableStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeVariableStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeVariableStatusChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Unlocked
            s.Write(value.Unlocked);
            //  Serialize TooltipStringId
            s.Write(value.TooltipStringId);
            //  Serialize InstanceId
            s.Write(value.InstanceId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeVariableStatusChanged)) as Rts.CnC.Messages.Client.TechTreeVariableStatusChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Unlocked
            s.Read(out value.Unlocked);
            //  Deserialize TooltipStringId
            s.Read(out value.TooltipStringId);
            //  Deserialize InstanceId
            s.Read(out value.InstanceId);

            return value;
        }
        
    }
}
