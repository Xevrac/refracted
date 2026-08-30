using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_TechTreeUnitAbilityStatusChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.TechTreeUnitAbilityStatusChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.TechTreeUnitAbilityStatusChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize Unlocked
            s.Write(value.Unlocked);
            //  Serialize TooltipStringId
            s.Write(value.TooltipStringId);
            //  Serialize InstanceId
            s.Write(value.InstanceId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.TechTreeUnitAbilityStatusChanged)) as Rts.CnC.Messages.Client.TechTreeUnitAbilityStatusChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
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
